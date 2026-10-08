.PHONY: build build-extension

build:
	bash src/compiler.bash src/scrippo.scrippo > scrippo

build-extension:
	cd dev/vscode-ext && npm run package