use std::fs;
use std::path::Path;
use std::time::SystemTime;

/// Проверяет, является ли файл бинарным (содержит нулевой байт в первых 1024 байтах).
pub fn is_binary(path: &Path, file_size: u64) -> bool {
    if file_size == 0 {
        return false;
    }
    let mut file = match fs::File::open(path) {
        Ok(f) => f,
        Err(_) => return false,
    };
    let mut buffer = vec![0; 1024];
    let n = match std::io::Read::read(&mut file, &mut buffer) {
        Ok(n) if n > 0 => n,
        _ => return false,
    };
    buffer[..n].contains(&0)
}

/// Читает файл как строку. Если `include_binary` истинно, читает как байты и заменяет невалидные последовательности.
pub fn read_file_content(path: &Path, include_binary: bool) -> anyhow::Result<String> {
    if include_binary {
        let bytes = fs::read(path)?;
        Ok(String::from_utf8_lossy(&bytes).to_string())
    } else {
        Ok(fs::read_to_string(path)?)
    }
}

/// Обрезает содержимое до указанного числа строк.
pub fn truncate_lines(content: &str, max_lines: usize) -> String {
    let lines: Vec<&str> = content.lines().take(max_lines).collect();
    let mut result = lines.join("\n");
    if content.lines().count() > max_lines {
        result.push_str("\n[...truncated...]");
    }
    result
}

/// Преобразует размер в человекочитаемый вид.
pub fn human_readable_size(bytes: u64) -> String {
    const UNITS: [&str; 6] = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
    if bytes == 0 {
        return "0 B".into();
    }
    let i = (bytes as f64).log2() / 10.0;
    let i = i.floor() as usize;
    let i = i.min(UNITS.len() - 1);
    let value = bytes as f64 / (1024u64.pow(i as u32) as f64);
    format!("{:.1} {}", value, UNITS[i])
}

/// Форматирует SystemTime в ISO 8601 (упрощённо).
pub fn format_time_iso(time: SystemTime) -> String {
    let datetime: chrono::DateTime<chrono::Local> = time.into();
    datetime.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}
