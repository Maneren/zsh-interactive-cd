install:
	cargo build --manifest-path=zic-list-dirs/Cargo.toml --release
	ln -s ./zic-list-dirs/target/release/zic-list-dirs ~/.local/bin/
	rm -rf ./zic-list-dirs/target
