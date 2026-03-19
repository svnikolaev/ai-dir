# ai-dir — быстрый анализ директорий с описанием файлов

**ai-dir** — это инструмент командной строки для анализа содержимого директорий. Он умеет:

- Извлекать ключевые символы (классы, функции, структуры) из файлов разных языков программирования (режим `pattern`).
- Генерировать однострочные описания с помощью AI‑моделей (режим `llm`).
- Выгружать полное содержимое файлов в одном потоке (режим `dump`) с разметкой plain, Markdown или XML.
- Показывать git‑diff (для подготовки коммитов).
- **Подсчитывать строки в файлах и отмечать функции, превышающие заданный порог** (по умолчанию 20 строк), помогая находить кандидатов на рефакторинг.
- **Гибко управлять выводом**: отключать обрезку списка символов или скрывать индикатор длинных функций.
- **Принудительно очищать кэш** для полного пересоздания описаний.

Утилита полезна для быстрого знакомства с проектом, создания документации, интеграции с LLM‑инструментами (`ai-assist`, `ai-shell`).

## Возможности

- **Два режима анализа**  
  - `pattern` — быстрый, офлайн, на основе регулярных выражений. Показывает общее количество строк в файле и помечает функции, превышающие порог (пока только для Rust).  
  - `llm` — через AI‑бэкенды (OpenAI‑совместимые) с fallback между несколькими.
- **Древовидный вывод** в форматах plain, цветной, JSON и XML.
- **Гибкая фильтрация** файлов через регулярные выражения `--include` / `--exclude`.
- **Уважение `.gitignore`** (можно отключить в конфиге).
- **Глобальное кэширование** результатов с проверкой времени изменения файла и учётом режима/параметров (разные режимы не затирают друг друга).
- **Управление кэшем**: `--refresh-cache` для полной очистки, `--no-cache` для временного отключения.
- **Режим `--dump`** для выгрузки содержимого в plain, Markdown или XML с метаданными.
- **Git‑diff** для индекса (`--staged`) или рабочей директории (`--diff`).
- **Настройка отображения**:  
  - `--no-truncate` — не обрезать список символов (для plain/color).  
  - `--no-long-indicator` — убрать индикатор `(long: N)` из основной строки.
  - `--format xml` — вывод дерева в XML.
- **Настраиваемый порог длинных функций**: `--long 40` покажет функции длиннее 40 строк (по умолчанию 20).
- **Конфигурация** в TOML (глобальная `~/.config/ai-dir/config.toml` и локальная `.ai-dir.toml`).

## Примеры использования

### Мгновенная оценка проекта для нового разработчика

```bash
aid
```

```text
.
├── Cargo.toml [30 lines] (table package, key name, …)
├── src/main.rs [109 lines] (struct Args, fn main) (long: 1)
└── src/commands/analyze.rs [165 lines] (fn count_lines, fn find_long_functions_rust, …) (long: 2)
```

**Результат:** новичок видит структуру проекта, ключевые функции и сразу понимает, где находится основная логика. А индикатор `(long: N)` подсвечивает сложные файлы.

### Найти большие функции для рефакторинга

```bash
aid --long
```

Вывод:

```text
src/commands/analyze.rs [165 lines]
  └── find_long_functions_rust: 42 lines
  └── describe_files_pattern: 99 lines

src/main.rs [109 lines]
  └── main: 101 lines
```

**Результат:** больше не нужно листать файлы вручную — вы сразу видите все функции, превышающие порог (20 строк), и их точный размер. Рефакторинг становится простым и быстрым.

### Подготовить идеальный контекст для ChatGPT (ревью кода)

```bash
aid --dump --include '\.(rs|md)$' --exclude target . > project_dump.txt
```

Теперь отправьте этот файл ChatGPT с промптом:
> "Проведи ревью этого Rust-проекта, выдели проблемные места и предложи улучшения"

**Результат:** модель получает полный контекст и даёт осмысленный ответ, а вам не нужно копировать файлы по одному.

### Автоматическое сообщение для коммита за секунду

```bash
aid -gs | llm "Напиши краткое сообщение коммита по этим изменениям"
```

**Результат:** git-diff отправляется прямо в LLM, и вы получаете готовое сообщение. Никакой головной боли с написанием commit message.

### Слепок проекта для документации или аудита

```bash
aid --dump xml --detailed --absolute-paths . > project_snapshot.xml
```

**Результат:** структурированный XML со всеми файлами, их размерами, датами изменения и полным содержимым. Можно передать в любую систему, поддерживающую XML, или сохранить для истории.

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

# Вывод в XML (структура проекта)
aid --format xml

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

# XML (полное содержимое файлов)
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

**Особенности:**
- Для каждого файла выводится общее количество строк в квадратных скобках: `[30 lines]`.
- Если в файле есть функции длиннее порога (по умолчанию 20 строк), рядом добавляется индикатор `(long: N)` (где N — количество таких функций).
- Список символов по умолчанию обрезается до 10 и помечается многоточием. Чтобы увидеть все символы, используйте `--no-truncate`.
- Для машиночитаемых форматов (JSON, XML) список символов всегда выводится полностью (без обрезки).

Пример вывода (plain):
```
.
├── Cargo.toml [30 lines] (table package, table dependencies, table dev-dependencies, key name, …)
├── LICENSE.txt [21 lines] (no symbols)
├── README.md [191 lines] (heading ai-dir, heading Возможности, …) (long: 2)
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

Флаг `--long` (или `-l`) выводит только те файлы, в которых есть функции длиннее заданного порога (по умолчанию 20 строк). Порог можно изменить, указав число:

```bash
# Функции длиннее 20 строк (по умолчанию)
aid --long

# Функции длиннее 40 строк
aid --long 40

# В JSON-формате
aid --long 50 --format json
```

Вывод содержит путь к файлу, общее количество строк и список длинных функций с точным размером:
```
src/commands/analyze/llm.rs [239 lines]
  └── describe_files: 73 lines
  └── call_backend: 31 lines
  └── test_llm_fallback: 41 lines

src/main.rs [109 lines]
  └── main: 101 lines
```

### Git‑diff для коммитов

Показывает изменения в рабочей директории или индексе. Удобно для генерации сообщений коммита.

```bash
# Неиндексированные изменения
aid --diff          # или aid -g

# Индексированные (staged)
aid --diff --staged # или aid -g -s, aid -gs
```

### Управление кэшем

Кэш хранится в `~/.cache/ai-dir/cache.json` и учитывает режим и параметры (например, для LLM — язык и модель).

- `--no-cache` — отключить использование кэша для текущего запуска.
- `--refresh-cache` (или `-R`) — полностью удалить файл кэша перед выполнением команды.

```bash
aid --refresh-cache          # очистить кэш и выполнить анализ заново
aid -R -m llm --lang ru      # очистить кэш и запустить llm-режим
```

### Управление выводом

- `--no-truncate` (или `-T`) — не обрезать список символов (для plain/color).
- `--no-long-indicator` — скрыть индикатор `(long: N)` в основной строке.
- `--format xml` — вывод дерева проекта в XML.

## Примеры использования

### Мгновенная оценка проекта для нового разработчика

```bash
aid
```

Вывод:
```
.
├── Cargo.toml [30 lines] (table package, key name, …)
├── src/main.rs [109 lines] (struct Args, fn main) (long: 1)
└── src/commands/analyze.rs [165 lines] (fn count_lines, fn find_long_functions_rust, …) (long: 2)
```

### Подготовка контекста для ChatGPT (ревью кода)

```bash
aid --dump > project_dump.txt
```

Затем можно отправить файл модели с промптом:
> "Проведи ревью этого Rust-проекта, выдели проблемные места и предложи улучшения"

### Автоматическое сообщение для коммита

```bash
aid -gs | llm "Напиши краткое сообщение коммита по этим изменениям"
```

### Слепок проекта для документации или аудита

```bash
aid --dump xml > project_snapshot.xml
```

### Поиск функций, требующих рефакторинга

```bash
aid --long 30 --format json > long_functions.json
```

## Интеграция с ai-assist

Пример настройки внешнего инструмента в `ai-assist`:

```toml
[[tools]]
name = "summarise_directory"
description = "Получить структуру директории с кратким описанием файлов."
schema = { type = "object", properties = { path = { type = "string" } }, required = ["path"] }

[tools.execution]
type = "command"
command = "aid {path} --format json"
```

Для получения только длинных функций:

```toml
command = "aid {path} --long --format json"
```

## Семейство AI-инструментов

- [ai-shell](https://github.com/svnikolaev/ai-shell) — AI-оболочка для командной строки
- [ai-assist](https://github.com/svnikolaev/ai-assist) — AI-помощник для разработки
- **ai-dir** — быстрый анализ директорий

## Лицензия

MIT © 2026
