.PHONY: build build-extension test

build:
	bash src/compiler.bash src/scrippo.scrippo > scrippo

test: build
	bash tests/unit/test_macros.bash
	bash tests/unit/test_shell.bash
	bash tests/unit/test_packages.bash

version:
	./scrippo dev/package.scrippo version $(word 2)

build-extension:
	cd dev/vscode-ext && npm run build

install:
	./install.sh
