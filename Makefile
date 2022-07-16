build:
	cargo build --manifest-path=zic-list-dirs/Cargo.toml --release
	mkdir -p ./bin
	mv ./zic-list-dirs/target/release/zic-list-dirs ./bin/
	rm -rf ./zic-list-dirs/target


download:
	echo "Downloading zic-list-dirs..."
	[ -d "bin" ] || mkdir bin
	TMP=$(shell mktemp -d /tmp/zic-list-dirs.XXXXXX); \
	LATEST_VERSION=$(shell git ls-remote --refs --sort="version:refname" --tags 2>/dev/null | cut -d/ -f3- | tail -n1); \
	ARCH=$(shell uname -m); \
	curl -L -o - \
		https://github.com/Maneren/zsh-interactive-cd/releases/download/$$LATEST_VERSION/zic-list-dirs_$$ARCH-unknown-linux-gnu.tar.xz \
	  | tar xvJf - --directory=$$TMP; \
	mv $$TMP/zic-list-dirs ./bin/zic-list-dirs


