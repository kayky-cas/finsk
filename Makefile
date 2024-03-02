TARGET = target/release/finsk
DEBUG_TARGET = target/debug/finsk

install: build
	sudo cp -f $(TARGET) /usr/local/bin/

install_debug: debug
	sudo cp -f $(DEBUG_TARGET) /usr/local/bin/

debug: src/main.rs
	cargo build

build: src/main.rs
	cargo build --release

clean:
	cargo clean
