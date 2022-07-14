build:
	cargo build --manifest-path=zic-list-dirs/Cargo.toml --release
	mkdir -p ./bin
	mv ./zic-list-dirs/target/release/zic-list-dirs ./bin/
	rm -rf ./zic-list-dirs/target

download:
	curl -L -o - \
		https://github.com/Maneren/zsh-interactive-cd/releases/download/$(shell git tag | tail -1)/zic-list-dirs_$(shell uname -m)-unknown-linux-gnu.tar.xz \
	        | tar xvJf - --directory=bin	

