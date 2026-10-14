.PHONY: build build-extension

build:
	cargo build
build-release:
	cargo build --release
test:
	cargo test
run:
	./target/debug/scriptopotamus
run-release:
	./target/release/scriptopotamus

version:
	./scrippo dev/package.scrippo version $(word 2)

build-extension:
	cd dev/vscode-ext && npm run build

install:
	./install.sh
