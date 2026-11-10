#!/bin/bash
# Runs `wasmtime "$@"`, printing its output and saving it to the file `$1`, and exits with how
# the run ended:
#
#   0  it ran to the end;
#   2  the main component trapped, e.g. a guest's panic on an error the fuzzed host returned,
#      which is what fuzzing finds, rather than a failure of the proxy;
#   1  anything else, a failure of the proxy: a trap in a generated component, which has no
#      name, or in the recorder, which reports the generated components' panics and replay's
#      divergences; or an error without a backtrace, e.g. "deadlock detected".
out=$1
shift
wasmtime "$@" 2>&1 | tee "$out"
[ "${PIPESTATUS[0]}" -eq 0 ] && exit 0
# The frame that trapped, the first of the backtrace.
frame=$(grep -m1 -A1 "wasm backtrace:" "$out" | tail -n +2)
case "$frame" in
  "" | *"<unknown>!"* | *"recorder.wasm!"* | *"recorder.one_byte_page.wasm!"*) exit 1 ;;
  *) exit 2 ;;
esac
