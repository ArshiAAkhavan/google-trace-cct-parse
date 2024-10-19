#!/bin/zsh

function bench {
  mode=$1
  trace=$2
  num_threads=$3

  # cache warmup:
  repeat 5 {RUST_LOG=ERROR ./target/release/experiment -n $num_threads --trace $trace --mode $mode 1>/dev/null 2>&1}

  # for i in $(seq 100)
  for i in $(seq 10)
  do
    out=$(RUST_LOG=ERROR ./target/release/experiment -n $num_threads --trace $trace --mode $mode 2>/dev/null)
    timers=$(echo "$out" | awk -F '):' '{print$2}')
    file_read=$(echo "$timers" | head -n1)
    cct_parse=$(echo "$timers" | tail -n1)
    echo "$mode($trace) [$num_threads]::Release : $file_read,$cct_parse"

    out=$(RUST_LOG=ERROR ./target/debug//experiment -n $num_threads --trace $trace --mode $mode 2>/dev/null)
    timers=$(echo "$out" | awk -F '):' '{print$2}')
    file_read=$(echo "$timers" | head -n1)
    cct_parse=$(echo "$timers" | tail -n1)
    echo "$mode($trace) [$num_threads]::Debug : $file_read,$cct_parse"
  done
}

$TRACE_DIRECTORY=$1

cargo build -p experiment
cargo build -p experiment --release


# threads=(1 2 4 8 16 32)
threads=(1 2 4 8)
traces=($(ls $TRACE_DIRECTORY))
modes=("baseline" "parallel-read" "parallel-parse" "parallel-cct")

for num_threads in "${threads[@]}"
do
  for mode in "${modes[@]}"
  do
    for trace in "${traces[@]}"
    do
      bench $mode $trace $num_threads
    done
  done
done
