.PHONY: build release clean install

build:
	cargo build --release

release:
	chmod +x build.sh
	./build.sh

clean:
	cargo clean
	rm -rf release/

install:
	cargo install --path .

# build for specific target
linux:
	cargo build --release --target x86_64-unknown-linux-gnu

windows:
	cross build --release --target x86_64-pc-windows-gnu

macos:
	cargo build --release --target x86_64-apple-darwin