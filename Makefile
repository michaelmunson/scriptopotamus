.PHONY: build build-release test run install sudo-install version build-extension install-nvim test-nvim

# `scrippo` is a scrippo script (cli/scrippo.scrippo) compiled to bash, wrapping the
# `scrippo-core` Rust binary that sits next to it.
define wrapper
target/$(1)/scrippo-core -c cli/scrippo.scrippo > target/$(1)/scrippo.tmp
chmod 755 target/$(1)/scrippo.tmp
mv target/$(1)/scrippo.tmp target/$(1)/scrippo
endef

build:
	cargo build
	$(call wrapper,debug)

build-release:
	cargo build --release
	$(call wrapper,release)

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

run: build
	target/debug/scrippo

install: build-release
	install -m 755 target/release/scrippo-core /usr/local/bin/scrippo-core
	install -m 755 target/release/scrippo /usr/local/bin/scrippo
	ln -sf scrippo /usr/local/bin/scriptopotamus

sudo-install: build-release
	sudo install -m 755 target/release/scrippo-core /usr/local/bin/scrippo-core
	sudo install -m 755 target/release/scrippo /usr/local/bin/scrippo
	sudo ln -sf scrippo /usr/local/bin/scriptopotamus