.PHONY: all build-components build-cli test

all: build-components build-cli
build-cli:
	cargo build --release
build-components:
	cargo build -p debug --target wasm32-wasip2
	cargo build -p recorder --target wasm32-wasip2

test:
	target/release/proxy-component instrument -m record tests/rust.wasm
	viceroy composed.wasm > trace.out &
	curl localhost:7676
	pkill -f viceroy || true
	cat trace.out
