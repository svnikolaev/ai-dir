# ai-dir — быстрый анализ директорий с описанием файлов

**ai-dir** — это инструмент командной строки для анализа содержимого директорий. Он умеет:

- Извлекать ключевые символы (классы, функции, структуры) из файлов разных языков программирования (режим `pattern`).
- Генерировать однострочные описания с помощью AI‑моделей (режим `llm`).
- Выгружать полное содержимое файлов в одном потоке (режим `dump`) с разметкой plain, Markdown или XML.
- Показывать git‑diff (для подготовки коммитов).
- **Подсчитывать строки в файлах и отмечать функции, превышающие порог (20 строк)**, помогая находить кандидатов на рефакторинг.
- **Гибко управлять выводом**: отключать обрезку списка символов или скрывать индикатор длинных функций.
- **Принудительно очищать кэш** для полного пересоздания описаний.

Утилита полезна для быстрого знакомства с проектом, создания документации, интеграции с LLM‑инструментами (`ai-assist`, `ai-shell`).

## Возможности

- **Два режима анализа**  
  - `pattern` — быстрый, офлайн, на основе регулярных выражений. Теперь показывает общее количество строк в файле и помечает функции длиннее 20 строк (пока только для Rust).  
  - `llm` — через AI‑бэкенды (OpenAI‑совместимые) с fallback между несколькими.
- **Древовидный вывод** (plain, цветной, JSON) с ограничением глубины.
- **Гибкая фильтрация** файлов через регулярные выражения `--include` / `--exclude`.
- **Уважение `.gitignore`** (можно отключить в конфиге).
- **Глобальное кэширование** результатов с проверкой времени изменения файла и учётом режима/параметров (разные режимы не затирают друг друга).
- **Управление кэшем**: `--refresh-cache` для полной очистки, `--no-cache` для временного отключения.
- **Режим `--dump`** для выгрузки содержимого в plain, Markdown или XML с метаданными.
- **Git‑diff** для индекса (`--staged`) или рабочей директории (`--diff`).
- **Настройка отображения**:
  - `--no-truncate` — не обрезать список символов (показывать все).
  - `--no-long-indicator` — убрать индикатор `(long: N)` из основной строки.
- **Полностью синхронный код** — предсказуемость и простота.
- **Конфигурация** в TOML (глобальная `~/.config/ai-dir/config.toml` и локальная `.ai-dir.toml`).

## Установка

```bash
# Из исходников
git clone https://github.com/svnikolaev/ai-dir
cd ai-dir
make build
sudo make install

# Или через cargo
cargo install --git https://github.com/svnikolaev/ai-dir
```

После установки создайте символьную ссылку для короткого вызова:

```bash
make link              # создаст ссылку `aid` в /usr/local/bin
# или вручную
sudo ln -s /usr/local/bin/ai-dir /usr/local/bin/aid
```

## Конфигурация

При первом запуске создаётся `~/.config/ai-dir/config.toml`. Пример:

```toml
default_mode = "pattern"

[[backends]]
name = "ollama"
api_url = "http://localhost:11434/v1/chat/completions"
api_key = ""
model = "qwen2.5:0.5b"
timeout_secs = 30

[backends.options]
temperature = 0.2

include_pattern = "\\.(rs|toml|md|txt|py|js|ts|go|java|cpp|c|h|cs|php|rb|swift|kt)$"
exclude_pattern = "(target|\\.git|node_modules|dist|build|\\.vscode|\\.idea|__pycache__|\\.venv|env)"
cache_enabled = false
language = "en"               # язык для LLM-описаний
respect_gitignore = true      # учитывать .gitignore
```

Можно создать `.ai-dir.toml` в корне проекта — он переопределит глобальные настройки.

## Использование

### Основные команды

```bash
# Анализ текущей директории (режим pattern)
aid

# Анализ с указанием пути
aid /path/to/project

# LLM-режим с русскими описаниями
aid -m llm --lang ru /path/to/project

# Ограничить глубину дерева
aid --depth 2

# Вывод в JSON
aid --format json

# Переопределить паттерны включения/исключения
aid --include '\.(py|js)$' --exclude '(venv|node_modules)'
```

### Режим дампа (`--dump`)

Выгружает содержимое файлов в одном потоке с разделителями. Полезно для передачи контекста LLM.

```bash
# Plain (по умолчанию)
aid --dump > dump.txt

# Markdown (можно сокращённо --dump md)
aid --dump markdown > docs.md

# XML
aid --dump xml > project.xml

# С метаданными (размер, дата) и абсолютными путями
aid --dump --detailed --absolute-paths . > full.txt

# Ограничить размер файла и количество строк
aid --dump --max-size 1M --max-lines 50 .

# Включить бинарные файлы (осторожно)
aid --dump --include-binary .

# Тихий режим (без предупреждений)
aid --dump --quiet .
```

Короткие флаги:

- `-D` = `--detailed`
- `-A` = `--absolute-paths`

### Режимы анализа

#### Pattern (по умолчанию)

Извлекает символы из файлов множества языков: Rust, Python, JavaScript, TypeScript, Go, Java, C/C++, C#, Ruby, Swift, Kotlin, TOML, YAML, JSON, Markdown и др.

**Новые возможности:**

- Для каждого файла выводится общее количество строк в квадратных скобках: `[30 lines]`.
- Если в файле есть функции длиннее 20 строк (пока только для Rust), рядом добавляется индикатор `(long: N)` (где N — количество таких функций), а сам список длинных функций можно увидеть отдельно с флагом `--long`.
- Список символов по умолчанию обрезается до 10 и помечается многоточием. Чтобы увидеть все символы, используйте `--no-truncate`.
- Чтобы убрать индикатор `(long: N)` из основной строки (например, если мешает), добавьте `--no-long-indicator`.

Пример вывода (plain):

```text
.
├── Cargo.toml [30 lines] (table package, table dependencies, table dev-dependencies, key name, …)
├── LICENSE.txt [21 lines] (no symbols)
├── README.md [191 lines] (heading ai-dir — быстрый анализ директорий, heading Возможности, …) (long: 2)
└── src/
    ├── commands/
    │   ├── analyze/
    │   │   ├── llm.rs [239 lines] (fn describe_files, fn call_backend, …) (long: 3)
    │   │   └── pattern.rs [96 lines] (struct MyStruct, enum MyEnum, fn create_file, …) (long: 1)
    │   └── ...
    └── main.rs [248 lines] (struct Args, fn main) (long: 1)
```

#### LLM

Отправляет содержимое файла на AI‑бэкенд и получает описание. Требует работающего сервера (Ollama, LM Studio, OpenAI и др.). Использует fallback‑бэкенды из конфига.

### Поиск длинных функций

Флаг `--long` (или `-l`) выводит только те файлы, в которых есть функции длиннее 20 строк, с детальным списком:

```bash
aid --long
```

Вывод:

```text
src/commands/analyze/llm.rs [239 lines]
  └── describe_files: 73 lines
  └── call_backend: 31 lines
  └── test_llm_fallback: 41 lines

src/commands/analyze/pattern.rs [96 lines]
  └── test_rust_patterns: 28 lines
```

### Управление выводом

- `--no-truncate` — не обрезать список символов (показывать все найденные символы, разделённые запятыми). Полезно для автоматической обработки.
- `--no-long-indicator` — скрыть индикатор `(long: N)` в основной строке. Сами длинные функции остаются доступны через `--long`.

### Git‑diff для коммитов

Показывает изменения в рабочей директории или индексе. Удобно для генерации сообщений коммита.

```bash
# Неиндексированные изменения
aid --diff          # или aid -g

# Индексированные (staged)
aid --diff --staged # или aid -g -s, aid -gs
```

### Работа с кэшем

Кэш хранится в `~/.cache/ai-dir/cache.json` и учитывает режим и параметры (например, для LLM — язык и модель). Это позволяет переключаться между режимами без потери данных.

- `--no-cache` — отключить использование кэша для текущего запуска.
- `--refresh-cache` (или `-R`) — полностью удалить файл кэша перед выполнением команды. Все файлы будут проанализированы заново.

```bash
aid --refresh-cache          # очистить кэш и выполнить анализ заново
aid -R -m llm --lang ru      # очистить кэш и запустить llm-режим
```

## Интеграция с ai-assist

Пример настройки внешнего инструмента в `ai-assist`:

```toml
[[tools]]
name = "summarise_directory"
description = "Получить структуру директории с кратким описанием каждого файла."
schema = { type = "object", properties = { path = { type = "string" } }, required = ["path"] }

[tools.execution]
type = "command"
command = "aid {path} --format json"
```

Если нужно передать дополнительные флаги (например, для обрезки символов или отключения индикатора), укажите их явно:

```toml
command = "aid {path} --format json --no-truncate"
```

## Семейство AI-инструментов

- [ai-shell](https://github.com/svnikolaev/ai-shell) — AI-оболочка для командной строки
- [ai-assist](https://github.com/svnikolaev/ai-assist) — AI-помощник для разработки
- **ai-dir** — быстрый анализ директорий

## Лицензия

MIT © 2026
