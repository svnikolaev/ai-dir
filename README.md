# ai-dir — быстрый анализ директорий с описанием файлов

**ai-dir** — это инструмент командной строки, который сканирует директорию и генерирует краткие однострочные описания для каждого файла. Поддерживает два режима:

- **LLM** — через один из нескольких настроенных AI‑бэкендов (OpenAI‑совместимые API).
- **pattern** — быстрое извлечение имён классов, функций, структур и других символов с помощью регулярных выражений (не требует API, работает офлайн).

Утилита полезна для быстрого ознакомления с незнакомым проектом, создания документации или интеграции с другими инструментами, такими как `ai-assist` и `ai-shell`.

## Возможности

- Поддержка нескольких AI‑бэкендов с автоматическим перебором при ошибках (fallback).
- Режим pattern для мгновенной работы без API.
- Глобальное кэширование результатов с проверкой времени изменения файла.
- Древовидный вывод в терминал (plain, цветной, JSON).
- Гибкая фильтрация файлов через регулярные выражения (include / exclude).
- Поддержка `.gitignore` через крейт `ignore`.
- Полностью синхронный и однопоточный код — простота и предсказуемость.
- Конфигурация в TOML-файле (глобальная и локальная для проекта).

## Установка

### Из исходников

```bash
git clone https://github.com/yourname/ai-dir
cd ai-dir
make build
sudo make install
```

### Через cargo (если репозиторий опубликован)

```bash
cargo install --git https://github.com/yourname/ai-dir
```

После установки проверьте:

```bash
ai-dir --help
```

По умолчанию бинарник называется `ai-dir`, но вы можете создать короткую ссылку:

```bash
make link LINK_NAME=aid   # или просто make link (по умолчанию aid)
```

## Конфигурация

При первом запуске создаётся глобальный конфигурационный файл `~/.config/ai-dir/config.toml` с настройками по умолчанию. Вы можете отредактировать его под свои нужды.

Для каждого проекта можно создать локальный конфиг `.ai-dir.toml` в корне проекта. Его параметры переопределяют глобальные.

```toml
# Режим по умолчанию: "llm" или "pattern"
default_mode = "pattern"

# Список бэкендов для LLM-режима (опрашиваются по порядку)
[[backends]]
name = "ollama"
api_url = "http://localhost:11434/v1/chat/completions"
api_key = ""
model = "qwen2.5:0.5b"
timeout_secs = 30

[backends.options]
temperature = 0.2

# Паттерны включения/исключения файлов (регулярные выражения)
include_pattern = "\\.(rs|toml|md|txt|py|js|ts|go|java|cpp|c|h|cs|php|rb|swift|kt)$"
exclude_pattern = "(target|\\.git|node_modules|dist|build|\\.vscode|\\.idea|__pycache__|\\.venv|env)"

# Настройки кэша
cache_enabled = false
language = "en"   # "en" или "ru"
```

## Использование

```bash
# Проанализировать текущую директорию в pattern-режиме (по умолчанию)
aid

# Указать путь как позиционный аргумент
aid /path/to/project

# Переключить режим на LLM
aid -m llm /path/to/project

# Ограничить глубину дерева
aid --depth 2 /path/to/project

# Вывод в JSON (для машинной обработки)
aid --format json /path/to/project

# Переопределить include/exclude
aid --include "\\.(py|js)$" --exclude "(venv|node_modules)"

# Игнорировать кэш
aid --no-cache

# Выбрать язык описаний (для LLM-режима)
aid -m llm --lang ru /path/to/project

# Показать справку
aid --help
```

### Режимы анализа

#### Режим pattern (по умолчанию)

Извлекает из исходных файлов символы для множества языков: Rust, Python, JavaScript, TypeScript, Go, Java, C/C++, C#, Ruby, Swift, Kotlin, TOML, YAML, JSON, Markdown и др.

Пример вывода:

```text
.
├── Cargo.toml  (table package, name, version, edition, dependencies)
├── README.md   (heading ai-dir — быстрый анализ директорий с описанием файлов)
└── src/
    ├── main.rs  (struct Args, fn main)
    ├── config.rs  (struct Config, impl Default, fn load)
    └── scanner.rs  (fn scan)
```

#### Режим LLM

Отправляет содержимое файла (или его начало) на указанный AI‑бэкенд и получает сгенерированное описание. Требует работающего сервера (локального или облачного), совместимого с OpenAI API (Ollama, LM Studio, OpenAI, OpenRouter и др.).

### Режим дампа содержимого

Флаг `--dump` позволяет выгрузить полное содержимое всех отфильтрованных файлов в один поток с разделителями. Это удобно для передачи контекста LLM или создания слепка проекта.

```bash
# Базовое использование
`ai-dir --dump --include '\.(rs|md)$' --exclude 'target' . > project.txt`

# С ограничением размера файла (например, не больше 1 MiB)
`ai-dir --dump --max-size 1048576 . > small_files.txt`

# Только первые 50 строк каждого файла
`ai-dir --dump --max-lines 50 src/ > preview.txt`

# Включить бинарные файлы (осторожно: может быть много мусора)
`ai-dir --dump --include-binary . > everything.txt`

# Тихий режим (без предупреждений в stderr)
`ai-dir --dump --quiet . > dump.txt`

#### Расширенные возможности дампа

- `--detailed` – добавляет размер и дату изменения в заголовки для форматов `plain` и `markdown`.
- `--absolute-paths` – выводит абсолютные пути вместо относительных.
- `--dump-format {plain|markdown|xml}` – выбирает формат вывода.
  - `plain` (по умолчанию) – текстовый формат с разделителями.
  - `markdown` – формат Markdown с блоками кода и метаданными.
  - `xml` – структурированный XML с метаданными в атрибутах.

Примеры:
```bash
# Plain с метаданными и абсолютными путями
`ai-dir --dump --detailed --absolute-paths . > project.txt`

# Markdown для документации
`ai-dir --dump --dump-format markdown src/ > docs.md`

# XML для интеграции
`ai-dir --dump --dump-format xml . > project.xml`

### Кэширование

Результаты анализа сохраняются в глобальном кэше (`~/.cache/ai-dir/cache.json`). При повторном запуске для неизменённых файлов используются закэшированные описания, что ускоряет работу. Кэш автоматически инвалидируется при изменении файла (по mtime). Кэш по умолчанию отключён; включите его в конфиге.

## Интеграция с ai-assist и ai-shell

В конфигурации `ai-assist` можно добавить внешний инструмент, вызывающий `ai-dir`:

```toml
[[tools]]
name = "summarise_directory"
description = "Получить структуру директории с кратким описанием каждого файла."
schema = { type = "object", properties = { path = { type = "string" } }, required = ["path"] }

[tools.execution]
type = "command"
command = "aid --path {path} --format json"
```

**Примечание:** Если вы используете короткую команду `aid`, не забудьте, что в примере выше путь передаётся как позиционный аргумент. Правильная команда для `ai-assist` будет:

```toml
command = "aid {path} --format json"
```

Или, если вы предпочитаете полное имя:

```toml
command = "ai-dir {path} --format json"
```

## Семейство AI-инструментов

- [ai-shell](https://github.com/svnikolaev/ai-shell) — AI-оболочка для командной строки
- [ai-assist](https://github.com/svnikolaev/ai-assist) — AI-помощник для разработки
- **ai-dir** — быстрый анализ директорий с описанием файлов

## Лицензия

MIT © 2026
