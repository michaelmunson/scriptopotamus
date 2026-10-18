.PHONY: build build-release test run install version build-extension

build:
	cargo build

build-release:
	cargo build --release

build-extension:
	cd ./dev/vscode-ext && rm -f ./*.vsix && npm run build

test:
	cargo test

run:
	cargo run --bin scrippo

install: build-release
	install -m 755 target/release/scrippo /usr/local/bin/scrippo
	ln -sf /usr/local/bin/scrippo /usr/local/bin/scriptopotamus

sudo-install: build-release
	install -m 755 target/release/scrippo /usr/local/bin/scrippo
	ln -sf /usr/local/bin/scrippo /usr/local/bin/scriptopotamus