use crate::core::cache::Cache;
use crate::core::config::{Backend, Config};
use crate::core::types::{Description, FileEntry};
use anyhow::{Result, anyhow};
use reqwest::blocking::Client;
use serde_json::{Value, json};
use std::fs;

pub fn describe_files(
    files: &[FileEntry],
    config: &Config,
    cache: &mut Cache,
) -> Result<Vec<Description>> {
    let client = Client::new();
    let mut results = Vec::new();

    for file in files {
        if let Some(cached) = cache.get(&file.path) {
            results.push(Description::cached(file, cached.to_string()));
            continue;
        }

        let content = match fs::read_to_string(&file.path) {
            Ok(c) => c,
            Err(e) => {
                results.push(Description::error(file, format!("cannot read: {}", e)));
                continue;
            }
        };
        let truncated = if content.len() > 8000 {
            format!("{}\n\n[...file truncated...]", &content[..8000])
        } else {
            content
        };

        let prompt = format!(
            "Describe the purpose of this {} file in one line (max 20 words) {}:\n\n{}",
            file.relative,
            config.language.prompt_suffix(),
            truncated
        );

        let mut success = false;
        for backend in &config.backends {
            match call_backend(&client, backend, &prompt) {
                Ok(text) => {
                    let desc = Description::new(file, text.clone());
                    cache.insert(file.path.clone(), text);
                    results.push(desc);
                    success = true;
                    break;
                }
                Err(e) => eprintln!("Backend '{}' failed: {}", backend.name, e),
            }
        }

        if !success {
            results.push(Description::error(file, "all backends failed".into()));
        }
    }

    Ok(results)
}

fn call_backend(client: &Client, backend: &Backend, prompt: &str) -> Result<String> {
    let mut body = json!({
        "model": backend.model,
        "messages": [{"role": "user", "content": prompt}],
        "max_tokens": 150,
        "temperature": 0.2,
    });
    if let Some(opts) = &backend.options {
        body["options"] = Value::Object(opts.clone());
    }

    let mut req = client.post(&backend.api_url).json(&body);
    if let Some(key) = &backend.api_key {
        req = req.bearer_auth(key);
    }
    let timeout = backend.timeout_secs.unwrap_or(30);
    req = req.timeout(std::time::Duration::from_secs(timeout));

    let resp = req.send()?;
    if !resp.status().is_success() {
        return Err(anyhow!("HTTP {}", resp.status()));
    }
    let json: Value = resp.json()?;
    let content = json["choices"][0]["message"]["content"]
        .as_str()
        .ok_or_else(|| anyhow!("no content in response"))?
        .trim()
        .to_string();
    Ok(content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::config::Backend;
    use crate::core::types::FileEntry;
    use mockito::Server;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    fn create_file(content: &str) -> (FileEntry, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let path = dir.path().join("test.txt");
        let mut f = File::create(&path).unwrap();
        write!(f, "{}", content).unwrap();
        let entry = FileEntry {
            path,
            relative: "test.txt".to_string(),
        };
        (entry, dir)
    }

    #[test]
    fn test_llm_success() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(r#"{"choices":[{"message":{"content":"This is a test file."}}]}"#)
            .create();

        let backend = Backend {
            name: "test".into(),
            api_url: format!("{}/v1/chat/completions", server.url()),
            api_key: None,
            model: "test-model".into(),
            timeout_secs: Some(5),
            options: None,
        };
        let config = crate::core::config::Config {
            backends: vec![backend],
            language: crate::core::types::Language::En,
            ..Default::default()
        };

        let (entry, _dir) = create_file("dummy content");
        let mut cache = Cache::new();
        let result = describe_files(&[entry], &config, &mut cache).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].text, "This is a test file.");
        mock.assert();
    }

    #[test]
    fn test_llm_fallback() {
        let mut server = Server::new();
        let mock1 = server
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .create();
        let mock2 = server
            .mock("POST", "/v2/chat/completions")
            .with_status(200)
            .with_body(r#"{"choices":[{"message":{"content":"Fallback response"}}]}"#)
            .create();

        let backend1 = Backend {
            name: "fail".into(),
            api_url: format!("{}/v1/chat/completions", server.url()),
            api_key: None,
            model: "fail".into(),
            timeout_secs: Some(1),
            options: None,
        };
        let backend2 = Backend {
            name: "success".into(),
            api_url: format!("{}/v2/chat/completions", server.url()),
            api_key: None,
            model: "success".into(),
            timeout_secs: Some(1),
            options: None,
        };
        let config = crate::core::config::Config {
            backends: vec![backend1, backend2],
            language: crate::core::types::Language::En,
            ..Default::default()
        };

        let (entry, _dir) = create_file("test");
        let mut cache = Cache::new();
        let result = describe_files(&[entry], &config, &mut cache).unwrap();
        assert_eq!(result[0].text, "Fallback response");
        mock1.assert();
        mock2.assert();
    }

    #[test]
    fn test_llm_all_fail() {
        let mut server = Server::new();
        let mock = server
            .mock("POST", "/v1/chat/completions")
            .with_status(500)
            .create();

        let backend = Backend {
            name: "fail".into(),
            api_url: format!("{}/v1/chat/completions", server.url()),
            api_key: None,
            model: "fail".into(),
            timeout_secs: Some(1),
            options: None,
        };
        let config = crate::core::config::Config {
            backends: vec![backend],
            language: crate::core::types::Language::En,
            ..Default::default()
        };

        let (entry, _dir) = create_file("test");
        let mut cache = Cache::new();
        let result = describe_files(&[entry], &config, &mut cache).unwrap();
        assert_eq!(result[0].text, "[ERROR]");
        assert!(result[0].error.as_deref() == Some("all backends failed"));
        mock.assert();
    }
}
