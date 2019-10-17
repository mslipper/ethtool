build-release:
	cargo build --release
	strip target/release/ethtool

install: build-release
	sudo cp target/release/ethtool /usr/local/bin

.PHONY: install