# A library and CLI for WIT component virtualization

This repository explores the concept of virtualizing and/or instrumenting WIT components. Given a Wasm component, 
we synthesize a new component that virtualizes the host interface and can optionally call the original host when necessary. 
This synthesized component can be linked, via `wac`, to produce a resulting component with the exact same WIT interface as the original, 
but with the added side effects from the virtualized component. This allows us to instrument or virtualize the Wasm component without modifying the user code.

Currently, the tool focuses on using this technique to perform record and replay for Wasm components. 
In the future, we can apply the same technique to other use cases, such as service chaining.

## Usage

### Record

```
$ cargo run instrument -m record <component.wasm>
```
Run `composed.wasm` in the host runtime which the original wasm is supposed to run. The runtime also needs to implement 
the [`record` interface](https://github.com/chenyan2002/proxy-component/blob/main/assets/recorder.wit#L3). See this [example PR](https://github.com/fastly/Viceroy/pull/546).

In the future, we can make the `record` interface as a component, so that we don't need to make any changes to the host runtime.

### Replay 

Assuming the trace captured from the record phase is stored in `trace.out`. We can run the following to replay the trace.

```
$ cargo run instrument -m replay <component.wasm>
$ cargo run run composed.wasm --invoke 'start()' --trace trace.out
```

Note that the trace is self-contained, and `composed.wasm` doesn't have any imports. This means that we can run `composed.wasm`
in a regular `wasmtime` without the host interface.

Another interesting use case is that we can replay the trace with a different Wasm binary, likely with a different compiler flag, or
a different optimization strategy, to compare the performance. We have assertions in the replay phase to make sure that the trace
is still valid with the new binary.

## Prerequisite

* rustup target add wasm32-unknown-unknown
* wasm-tools
* wit-bindgen
* wac
