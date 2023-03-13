build:
	cargo build --manifest-path=zic-list-dirs/Cargo.toml --release
	mkdir -p ./bin
	mv ./zic-list-dirs/target/release/zic-list-dirs ./bin/
	rm -rf ./zic-list-dirs/target

dev:
	cargo build --manifest-path=zic-list-dirs/Cargo.toml
	mkdir -p ./bin
	mv ./zic-list-dirs/target/debug/zic-list-dirs ./bin/

download: REPO:=https://github.com/Maneren/zsh-interactive-cd
download: LATEST_VERSION:=$(shell git ls-remote --refs --sort="version:refname" --tags 2>/dev/null | cut -d/ -f3- | tail -n1)
download: NAME:=zic-list-dirs_$(shell uname -m)-unknown-linux-gnu.tar.xz
download: URL:=$(REPO)/releases/download/$(LATEST_VERSION)/$(NAME)
download:
	@echo "Downloading zic-list-dirs..."
	[ -d "bin" ] || mkdir bin
	curl -L -o - "$(URL)" | tar xJf - --directory="./bin"
