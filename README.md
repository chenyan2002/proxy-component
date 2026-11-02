# A library and CLI for WIT component virtualization

This repository explores the concept of virtualizing and/or instrumenting WIT components. Given a Wasm component, 
we synthesize a new component that virtualizes the host interface and can optionally call the original host when necessary. 
This synthesized component can be linked, via `wac`, to produce a resulting component with the exact same WIT interface as the original, 
but with the added side effects from the virtualized component. This allows us to instrument or virtualize the Wasm component without modifying the user code, nor the host runtime.

Currently, the tool focuses on using this technique to perform fuzzing, and record & replay for Wasm components.
In the future, we can apply the same technique to other use cases, such as generating adapters.

## Build

To build the CLI tool, just run `make all`.

To test record and fuzzing, run `make test`.

`cargo test` checks the generated sources against the snapshots in `tests/snapshots`, generated
from the WIT in `tests/fixtures/proxy`. After an intended change, run `BLESS=1 cargo test` and
review the snapshots' diff.

## Usage

### Record

```
$ proxy-component instrument -m record <component.wasm>
$ <your_wasm_runtime> composed.wasm > trace.out  # store the stdout trace to trace.out
```
Run `composed.wasm` in the host runtime which the original wasm is supposed to run. The tool provides a [guest implementation](components/recorder/) for record and replay APIs, which outputs the trace to stdout while recording, and reads the trace from stdin while replay.

The host runtime can also choose to implement the [`record` interface](https://github.com/chenyan2002/proxy-component/blob/main/assets/recorder.wit#L3). Then we can use the `--use-host-recorder` flag to skip composing the guest-side record implementation.

Note that the recorded handle id is not an exact match with the handle id the main component sees. Each
component instance has its own handle table, and the main component can allocate handles in it
that the proxy never sees, e.g., its own resources and streams. Each of those
shifts the main component's handle ids, so the imports component's handle ids, from its own table,
can't match them exactly.

### Replay

Assuming the trace captured from the record phase is stored in `trace.out`. We can run the following to replay the trace.

```
$ proxy-component instrument -m replay <component.wasm>
$ wasmtime --invoke 'start()' composed.wasm < trace.out
```

Note that the trace is self-contained, and `composed.wasm` doesn't have any imports. This means that we can run `composed.wasm` in a regular `wasmtime`.

Another interesting use case is that we can replay the trace with a different Wasm binary, likely with a different compiler flag, or
a different optimization strategy, to compare the performance. 

We provide a [Debug component](components/debug/) that does not go through instrumentation. You can use the Debug component in your code to perform I/O operations while in the replay mode. We have assertions in the replay phase to make sure that the trace
is still valid with the new binary.

### Fuzzing

```
$ proxy-component instrument -m fuzz <component.wasm>
$ wasmtime --invoke 'start()' composed.wasm
```

Fuzzing the import return and export input based on the WIT type. This mode requires a [Debug component](components/debug/) to get random numbers and logging. The `composed.wasm` can be run in
a standalone `wasmtime` without any special host functions.

### Dialog

```
$ proxy-component instrument -m dialog <component.wasm>
$ proxy-component run composed.wasm --invoke 'start()'
```

Provide an interactive terminal for users to mock import and export calls.
[![asciicast](https://asciinema.org/a/BeVtK4cwsTsqmDo6.svg?render=1)](https://asciinema.org/a/BeVtK4cwsTsqmDo6)

This mode requires raw access to terminal, which can only be implemented on the host side for now.
So the composed binary can only be run with `proxy-component run`, instead of a regular `wasmtime`.

## Prerequisite

* rustup target add wasm32-unknown-unknown
* wasm-tools
* wit-bindgen
* wac
* viceroy (only needed to run the record test suite)
