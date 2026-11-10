# The spike of docs/p3-record-replay.md. `make` builds, composes, and runs ./run.sh, which records
# each scenario and checks that its replay matches. `make component-host` records with the host as
# another component instead of wasmtime's own host functions, which replay can't reproduce.
R = target/wasm32-wasip2/release
RC = target/component-host/wasm32-wasip2/release
.PHONY: all build compose component-host
all: compose
	./run.sh 20
build:
	cargo build --release --target wasm32-wasip2
	cargo build --release --target wasm32-wasip2 -p race-proxy-record --features component-host --target-dir target/component-host
compose: build
	wac compose compose-record.wac --dep spike:proxy=$(R)/race_proxy_record.wasm --dep spike:guest=$(R)/race_guest.wasm --dep spike:driver=$(R)/race_driver.wasm -o record.wasm
	wac compose compose-record-component-host.wac --dep spike:host=$(R)/race_host.wasm --dep spike:proxy=$(RC)/race_proxy_record.wasm --dep spike:guest=$(R)/race_guest.wasm --dep spike:driver=$(R)/race_driver.wasm -o record-component-host.wasm
	wac compose compose-replay.wac --dep spike:proxy=$(R)/race_proxy_replay.wasm --dep spike:guest=$(R)/race_guest.wasm --dep spike:driver=$(R)/race_driver.wasm -o replay.wasm
component-host: compose
	RECORD=record-component-host.wasm OUT=target/runs-component-host ./run.sh 20
