.PHONY: all build-components build-cli test test-record test-fuzz test-dialog run-fuzz run-record run-dialog run-viceroy

all: build-components build-cli
build-cli:
	cargo build --all-features --release
build-components:
	cargo build -p debug --target wasm32-wasip2 --release
	cargo build -p recorder --target wasm32-wasip2 --release
	cp target/wasm32-wasip2/release/debug.wasm assets/debug.wasm
	cp target/wasm32-wasip2/release/recorder.wasm assets/recorder.wasm

test: test-fuzz test-record test-dialog

test-fuzz:
	$(MAKE) run-fuzz WASM=tests/calculator.wasm
	# build-only test
	target/release/proxy-component instrument -m fuzz tests/rust.wasm
	target/release/proxy-component instrument -m fuzz tests/go.wasm
	target/release/proxy-component instrument -m fuzz tests/python.wasm

test-record:
	$(MAKE) run-record WASM=tests/go.wasm
	$(MAKE) run-record WASM=tests/python.wasm
	$(MAKE) run-record WASM=tests/rust.wasm
	# test the same trace with a different wasm replay
	target/release/proxy-component instrument -m replay tests/rust.debug.wasm
	wasmtime --invoke 'start()' composed.wasm < trace.out
	# build-only test
	target/release/proxy-component instrument -m record tests/calculator.wasm
	target/release/proxy-component instrument -m replay tests/calculator.wasm

test-dialog:
	rm tests/composed.wasm || true
	for wasm in tests/*.wasm; do \
		$(MAKE) run-dialog WASM=$$wasm; \
	done

run-fuzz:
	target/release/proxy-component instrument -m fuzz $(WASM)
	wasmtime --invoke 'start()' composed.wasm

run-record:
	target/release/proxy-component instrument -m record $(WASM)
	$(MAKE) run-viceroy URL=localhost:7676
	target/release/proxy-component instrument -m replay $(WASM)
	wasmtime --invoke 'start()' composed.wasm < trace.out
	# test host replay
	target/release/proxy-component instrument -m replay --use-host-recorder $(WASM)
	target/release/proxy-component run composed.wasm --invoke 'start()' --trace trace.out

run-dialog:
	target/release/proxy-component instrument -m dialog $(WASM)
	# build-only
	# target/release/proxy-component run composed.wasm --invoke 'start()'

run-viceroy:
	viceroy composed.wasm > trace.out & echo $$! > viceroy.pid
	until nc -z localhost 7676; do \
		kill -0 $$(cat viceroy.pid) 2>/dev/null || exit 1; \
		sleep 1; \
	done
	curl $(URL)
	kill $$(cat viceroy.pid) || true
	rm viceroy.pid