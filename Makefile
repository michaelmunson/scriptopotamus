.PHONY: build build-extension

build:
	bash src/compiler.bash src/scrippo.scrippo > scrippo

version:
	./scrippo dev/package.scrippo version $(word 2)

build-extension:
	cd dev/vscode-ext && npm run build

install:
	./install.sh
