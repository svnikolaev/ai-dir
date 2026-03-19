# Makefile для ai-dir

# Имя символической ссылки (по умолчанию aid)
LINK_NAME ?= aid

.PHONY: all test build install link clean uninstall

all: test build install link

test: check-fmt
	cargo test

build:
	cargo build --release

install:
	cargo install --path .

link:
	@if [ -f ~/.cargo/bin/ai-dir ]; then \
		ln -sf ~/.cargo/bin/ai-dir ~/.cargo/bin/$(LINK_NAME); \
		echo "Ссылка создана: ~/.cargo/bin/$(LINK_NAME) -> ~/.cargo/bin/ai-dir"; \
	else \
		echo "Бинарник ai-dir не найден в ~/.cargo/bin. Сначала выполните make install."; \
		exit 1; \
	fi

clean:
	cargo clean

uninstall:
	@rm -f ~/.cargo/bin/ai-dir ~/.cargo/bin/$(LINK_NAME)
	@echo "Удалены ai-dir и $(LINK_NAME) из ~/.cargo/bin (если существовали)."

fmt:
	cargo sort
	cargo fmt --all

check-fmt:
	cargo fmt --all -- --check

ci: check-fmt test build