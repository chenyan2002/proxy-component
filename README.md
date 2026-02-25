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

## Usage

### Record

```
$ proxy-component instrument -m record <component.wasm>
$ <your_wasm_runtime> composed.wasm > trace.out  # store the stdout trace to trace.out
```
Run `composed.wasm` in the host runtime which the original wasm is supposed to run. The tool provides a [guest implementation](components/recorder/) for record and replay APIs, which outputs the trace to stdout while recording, and reads the trace from stdin while replay.

The host runtime can also choose to implement the [`record` interface](https://github.com/chenyan2002/proxy-component/blob/main/assets/recorder.wit#L3). Then we can use the `--use-host-recorder` flag to skip composing the guest-side record implementation.

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

### Generate

Given a `bindings.rs` file generated from `wit-bindgen`. This command can generate code to implement
all the required traits, based on the following mode:

* `stubs`. Fill in all impl functions with `unimplemented!()`, similar to `wit-bindgen rust --stubs`, but outside of the bindings module.
* `instrument`. Given an instrument component which imports and exports the same interface, generate code to redirect export interface to call the coressponding import functions.
* `record`. Given an instrument component, generate the code to redirect the calls and record the arguments and return in WAVE format. 
* `replay`. Given a vitualized component, generate code to replay an execution based on a recorded WAVE trace.
* `fuzz`. Given a virtualized component, generate random import values and export values using the `arbitrary` crate.

```
$ cargo run generate bindings.rs <mode> -o lib.rs
```

## Prerequisite

* rustup target add wasm32-unknown-unknown
* wasm-tools
* wit-bindgen
* wac
* viceroy (only needed to run the record test suite)
