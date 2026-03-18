use crate::cache::Cache;
use crate::config::{Backend, Config};
use crate::types::{Description, FileEntry};
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
