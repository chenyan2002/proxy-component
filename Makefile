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
  for i in $(seq 1 30); do
    nc -z localhost 7676 && break
    sleep 1
  done
	curl localhost:7676
	pkill -f viceroy || true
	cat trace.out
