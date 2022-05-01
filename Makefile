install:
	cargo build --manifest-path=zic-list-dirs/Cargo.toml --release
	mkdir -p ./bin
	mv ./zic-list-dirs/target/release/zic-list-dirs ./bin/
	rm -rf ./zic-list-dirs/target
