TARGET=iptools.so

$(TARGET): src/lib.rs src/avltree.rs
	cargo build --release
	cp target/release/libiptools.so $@

test: $(TARGET)
	cargo test
	lua test.lua

.PHONY: test

