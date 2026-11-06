.PHONY: all build-components build-cli test test-cargo test-record test-fixture test-fuzz-replay test-fuzz test-dialog run-fuzz run-fuzz-replay run-record run-dialog run-viceroy

# `make test ONE_BYTE_PAGE=1` tests with 1-byte-page memories
ifeq ($(ONE_BYTE_PAGE),1)
INSTRUMENT_FLAGS = --one-byte-page-size
WASMTIME_FLAGS = -W custom-page-sizes=y
endif
INSTRUMENT = target/release/proxy-component instrument $(INSTRUMENT_FLAGS)
WASMTIME = wasmtime $(WASMTIME_FLAGS)
# Runs wasmtime, and exits with 2 if the main component trapped, rather than the proxy failed
# (see tests/run-wasm.sh)
RUN_WASM = tests/run-wasm.sh

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

test: test-cargo test-fuzz test-record test-fixture test-fuzz-replay test-dialog

# Snapshot tests of the generated sources
test-cargo:
	cargo test

# A WASI p3 main component of tests/fixtures/proxy-p3, which uses error-context
P3_GUEST = tests/fixtures/proxy-p3-run/guest
P3_GUEST_OUT = target/fixture-p3
P3_GUEST_WASM = $(P3_GUEST_OUT)/wasm32-wasip2/release/proxy_p3_guest.wasm
test-fuzz:
	$(MAKE) run-fuzz WASM=tests/calculator.wasm
	$(MAKE) run-fuzz WASM=tests/wasi_http.wasm
	$(MAKE) run-fuzz WASM=tests/wasi_http_p3.wasm
	cargo build --manifest-path $(P3_GUEST)/Cargo.toml --target wasm32-wasip2 --release --target-dir $(P3_GUEST_OUT)
	$(MAKE) run-fuzz WASM=$(P3_GUEST_WASM) WASMTIME_FLAGS="$(WASMTIME_FLAGS) -W component-model-error-context=y"
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

# Records and replays the main component of tests/fixtures/proxy-run/guest, whose resources are
# dropped by the guest and by the host, in a composition with a host and a driver (see compose.wac)
FIXTURE = tests/fixtures/proxy-run
FIXTURE_OUT = target/fixture
FIXTURE_WASM = $(FIXTURE_OUT)/wasm32-wasip2/release
test-fixture:
	for crate in guest host driver; do \
		cargo build --manifest-path $(FIXTURE)/$$crate/Cargo.toml --target wasm32-wasip2 --release --target-dir $(FIXTURE_OUT) || exit 1; \
	done
	$(INSTRUMENT) -m record $(FIXTURE_WASM)/proxy_guest.wasm
	wac compose $(FIXTURE)/compose.wac -d test:mockhost=$(FIXTURE_WASM)/proxy_host.wasm -d test:driver=$(FIXTURE_WASM)/proxy_driver.wasm -d test:main=composed.wasm -o $(FIXTURE_OUT)/record.wasm
	$(WASMTIME) --invoke 'run()' $(FIXTURE_OUT)/record.wasm > $(FIXTURE_OUT)/trace.out
	# the host's drops of the guest's sessions
	grep -c '"by":"Host","at":null,"event":{"Drop"' $(FIXTURE_OUT)/trace.out | grep -qx 2
	$(INSTRUMENT) -m replay $(FIXTURE_WASM)/proxy_guest.wasm
	$(WASMTIME) --invoke 'start()' composed.wasm < $(FIXTURE_OUT)/trace.out
	$(INSTRUMENT) -m replay --use-host-recorder $(FIXTURE_WASM)/proxy_guest.wasm
	target/release/proxy-component run composed.wasm --invoke 'start()' --trace $(FIXTURE_OUT)/trace.out

# The spike's guest, which joins two concurrent results in the order its wakeups receive them
RACE = tests/fixtures/spike-p3-race
RACE_OUT = target/fixture-race
RACE_WASM = $(RACE_OUT)/wasm32-wasip2/release/race_guest.wasm
# Fuzzes the main components of tests/fixtures/proxy-run/guest, the p3 fixture, wasi_http_p3 and
# the spike's guest, and replays each trace it writes, which ends as the fuzzing run did: to the
# end, or in the main component's trap, e.g. a guest's panic on a fuzzed error.
test-fuzz-replay:
	cargo build --manifest-path $(FIXTURE)/guest/Cargo.toml --target wasm32-wasip2 --release --target-dir $(FIXTURE_OUT)
	$(MAKE) run-fuzz-replay WASM=$(FIXTURE_WASM)/proxy_guest.wasm OUT=$(FIXTURE_OUT)
	cargo build --manifest-path $(P3_GUEST)/Cargo.toml --target wasm32-wasip2 --release --target-dir $(P3_GUEST_OUT)
	$(MAKE) run-fuzz-replay WASM=$(P3_GUEST_WASM) OUT=$(P3_GUEST_OUT) WASMTIME_FLAGS="$(WASMTIME_FLAGS) -W component-model-error-context=y"
	$(MAKE) run-fuzz-replay WASM=tests/wasi_http_p3.wasm OUT=target/fixture-http-p3
	cargo build --manifest-path $(RACE)/Cargo.toml -p race-guest --target wasm32-wasip2 --release --target-dir $(RACE_OUT)
	$(MAKE) run-fuzz-replay WASM=$(RACE_WASM) OUT=$(RACE_OUT)
run-fuzz-replay:
	mkdir -p $(OUT)
	$(INSTRUMENT) -m fuzz $(WASM)
	cp composed.wasm $(OUT)/fuzz.wasm
	$(INSTRUMENT) -m replay $(WASM)
	cp composed.wasm $(OUT)/replay.wasm
	for i in 1 2 3 4 5 6 7 8 9 10; do \
		$(RUN_WASM) $(OUT)/fuzz-trace.out $(WASMTIME_FLAGS) --invoke 'start()' $(OUT)/fuzz.wasm > /dev/null; fuzz=$$?; \
		[ $$fuzz -ne 1 ] || exit 1; \
		$(RUN_WASM) $(OUT)/replay.out $(WASMTIME_FLAGS) --invoke 'start()' $(OUT)/replay.wasm < $(OUT)/fuzz-trace.out > /dev/null; \
		[ $$? -eq $$fuzz ] || exit 1; \
	done

test-dialog:
	rm tests/composed.wasm || true
	for wasm in tests/*.wasm; do \
		$(MAKE) run-dialog WASM=$$wasm || exit 1; \
	done
	cargo build --manifest-path $(P3_GUEST)/Cargo.toml --target wasm32-wasip2 --release --target-dir $(P3_GUEST_OUT)
	$(MAKE) run-dialog WASM=$(P3_GUEST_WASM)

# A trap of the main component is what fuzzing finds, not a failure.
run-fuzz:
	$(INSTRUMENT) -m fuzz $(WASM)
	$(RUN_WASM) /tmp/composed.out $(WASMTIME_FLAGS) --invoke 'start()' composed.wasm; [ $$? -ne 1 ]

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
