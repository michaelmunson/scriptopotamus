.PHONY: build build-release test run install version build-extension install-nvim test-nvim

build:
	cargo build

build-release:
	cargo build --release

build-extension:
	cd ./dev/vscode-ext && rm -f ./*.vsix && npm run build

test:
	cargo test

test-nvim:
	nvim --headless -u NONE -l dev/nvim/test/run.lua

NVIM_PACK_DIR ?= $(or $(XDG_DATA_HOME),$(HOME)/.local/share)/nvim/site/pack/scriptopotamus/start

install-nvim:
	mkdir -p $(NVIM_PACK_DIR)
	ln -sfn $(CURDIR)/dev/nvim $(NVIM_PACK_DIR)/scrippo.nvim

run:
	cargo run --bin scrippo

install: build-release
	install -m 755 target/release/scrippo /usr/local/bin/scrippo
	ln -sf /usr/local/bin/scrippo /usr/local/bin/scriptopotamus

sudo-install: build-release
	sudo install -m 755 target/release/scrippo /usr/local/bin/scrippo
	sudo ln -sf /usr/local/bin/scrippo /usr/local/bin/scriptopotamus