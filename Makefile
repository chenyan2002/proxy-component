.PHONY: all build-components build-cli test test-record test-fuzz test-dialog run-fuzz run-record run-dialog run-viceroy

# `make test ONE_BYTE_PAGE=1` tests with 1-byte-page memories
ifeq ($(ONE_BYTE_PAGE),1)
INSTRUMENT_FLAGS = --one-byte-page-size
WASMTIME_FLAGS = -W custom-page-sizes=y
endif
INSTRUMENT = target/release/proxy-component instrument $(INSTRUMENT_FLAGS)
WASMTIME = wasmtime $(WASMTIME_FLAGS)

all: build-components build-cli
build-cli:
	cargo build --all-features --release
# The CLI embeds both regular and 1-byte-page builds of these components.
build-components:
	cargo build -p debug -p recorder --target wasm32-wasip2 --release
	cargo build -p debug -p recorder --target wasm32-wasip2 --release --features debug/one-byte-page,recorder/one-byte-page --target-dir target/one-byte-page
	cp target/wasm32-wasip2/release/debug.wasm assets/debug.wasm
	cp target/wasm32-wasip2/release/recorder.wasm assets/recorder.wasm
	cp target/one-byte-page/wasm32-wasip2/release/debug.wasm assets/debug.one_byte_page.wasm
	cp target/one-byte-page/wasm32-wasip2/release/recorder.wasm assets/recorder.one_byte_page.wasm

test: test-fuzz test-record test-dialog

test-fuzz:
	$(MAKE) run-fuzz WASM=tests/calculator.wasm
	$(MAKE) run-fuzz WASM=tests/wasi_http.wasm
	# build-only test
	$(INSTRUMENT) -m fuzz tests/rust.wasm
	$(INSTRUMENT) -m fuzz tests/go.wasm
	$(INSTRUMENT) -m fuzz tests/python.wasm

test-record:
	$(MAKE) run-record WASM=tests/go.wasm
	$(MAKE) run-record WASM=tests/python.wasm
	$(MAKE) run-record WASM=tests/rust.wasm
	# test the same trace with a different wasm replay
	$(INSTRUMENT) -m replay tests/rust.debug.wasm
	$(WASMTIME) --invoke 'start()' composed.wasm < trace.out
	# build-only test
	$(INSTRUMENT) -m record tests/calculator.wasm
	$(INSTRUMENT) -m replay tests/calculator.wasm
	$(INSTRUMENT) -m record tests/wasi_http.wasm
	$(INSTRUMENT) -m replay tests/wasi_http.wasm

test-dialog:
	rm tests/composed.wasm || true
	for wasm in tests/*.wasm; do \
		$(MAKE) run-dialog WASM=$$wasm; \
	done

run-fuzz:
	$(INSTRUMENT) -m fuzz $(WASM)
	$(WASMTIME) --invoke 'start()' composed.wasm

run-record:
	$(INSTRUMENT) -m record $(WASM)
	$(MAKE) run-viceroy URL=localhost:7676
	$(INSTRUMENT) -m replay $(WASM)
	$(WASMTIME) --invoke 'start()' composed.wasm < trace.out
	# test host replay
	$(INSTRUMENT) -m replay --use-host-recorder $(WASM)
	target/release/proxy-component run composed.wasm --invoke 'start()' --trace trace.out

run-dialog:
	$(INSTRUMENT) -m dialog $(WASM)
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
