.PHONY: all build-components build-cli test test-record test-fuzz run-fuzz run-record run-viceroy

all: build-components build-cli
build-cli:
	cargo build --release
build-components:
	cargo build -p debug --target wasm32-wasip2
	cargo build -p recorder --target wasm32-wasip2

test: test-fuzz test-record

test-fuzz:
	RUSTFLAGS="" $(MAKE) run-fuzz WASM=tests/calculator.wasm

test-record:
	$(MAKE) run-record WASM=tests/rust.wasm
	$(MAKE) run-record WASM=tests/go.wasm
	$(MAKE) run-record WASM=tests/python.wasm

run-fuzz:
	target/release/proxy-component instrument -m fuzz $(WASM)
	wasmtime --invoke 'start()' composed.wasm

run-record:
	target/release/proxy-component instrument -m record $(WASM)
	$(MAKE) run-viceroy URL=localhost:7676
	target/release/proxy-component instrument -m replay $(WASM)
	wasmtime --invoke 'start()' composed.wasm < trace.out

run-viceroy:
	viceroy composed.wasm > trace.out & echo $$! > viceroy.pid
	until nc -z localhost 7676; do \
		kill -0 $$(cat viceroy.pid) 2>/dev/null || exit 1; \
		sleep 1; \
	done
	curl $(URL)
	kill $$(cat viceroy.pid) || true
	rm viceroy.pid