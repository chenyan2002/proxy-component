#!/bin/sh
# Records each scenario, replays its trace, and checks that the replay sees the same result and
# the same trace, line by line. Usage: ./run.sh [random runs]
# RECORD=record-component-host.wasm records with the host as a component instead.
set -u
OUT=${OUT:-target/runs}
mkdir -p "$OUT"
fail=0
check() { # name, then wasmtime --env flags
    name=$1; shift
    wasmtime run "$@" --invoke 'run()' "${RECORD:-record.wasm}" > "$OUT/$name.record" 2>&1
    wasmtime run --invoke 'run()' replay.wasm < "$OUT/$name.record" > "$OUT/$name.replay" 2>&1
    recorded=$(tail -1 "$OUT/$name.record"); replayed=$(tail -1 "$OUT/$name.replay")
    # Whether the guest got both results in one wakeup.
    together=$(grep -c 'wakeups: [A-Z][A-Z])' "$OUT/$name.record")
    shape=$(grep -E '^trace: ' "$OUT/$name.record" | sed 's/^trace: //; s/ [A-Z]*$//' | tr '\n' ',' )
    if cmp -s "$OUT/$name.record" "$OUT/$name.replay"; then
        echo "ok   $name: $recorded$([ "$together" -gt 0 ] && echo ' (the guest got both at once)') [$shape]"
    else
        echo "FAIL $name: recorded $recorded, replayed $replayed"
        diff "$OUT/$name.record" "$OUT/$name.replay" | head -10
        fail=1
    fi
}
check a-first     --env DELAY_A=5  --env DELAY_B=15
check b-first     --env DELAY_A=15 --env DELAY_B=5
check a-returned  --env DELAY_A=0  --env DELAY_B=10
check b-returned  --env DELAY_A=10 --env DELAY_B=0
check both-returned --env DELAY_A=0 --env DELAY_B=0
check same-delay  --env DELAY_A=5  --env DELAY_B=5
# The guest stays busy until both results have arrived.
check busy-a-first --env DELAY_A=5  --env DELAY_B=10 --env BUSY_MS=40
check busy-b-first --env DELAY_A=10 --env DELAY_B=5  --env BUSY_MS=40
check busy-same    --env DELAY_A=5  --env DELAY_B=5  --env BUSY_MS=40
i=0
while [ $i -lt "${1:-20}" ]; do
    check random-$i
    i=$((i + 1))
done
exit $fail
