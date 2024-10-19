# Google Trace CCT Parser

## Quick Start

make sure to have rust installed on your machine
```bash
rustc --version
```

clone the repository.
```bash
git clone https://github.com/ArshiAAkhavan/google-trace-cct-parse cct
cd cct
```
you can now parse the tracefile using different implementations.
```bash
cargo run -- -n <NUM_THREADS> --trace /path/to/tracefile --mode <implementation-name>
```
or for outmost performance, use the release profile.
```bash
cargo run --release -- -n <NUM_THREADS> --trace /path/to/tracefile --mode <implementation-name>
```
to run tests
```bash
cargo test
```
to run benchmark you can use the bench script.
```bash
./run-bench.sh path/to/trace/directory > bench-out.txt
```
to visualize the output you can use the visualize python script
```bash
python3 visualize.py bench-out.txt
```
## Introduction

This is an experiment that parses Google's trace file and optimizes the process.

the code takes several passes to the problem. each pass has its own optimizations and can be run seperatly.

### baseline:
Baseline is the sequential mode in which data is read sequentially, parsed
sequentially and calling context trees are formed sequentially.

to run, use the following command:
```bash
cargo run -- -n <NUM_THREADS> --trace /path/to/tracefile --mode baseline
```

### parallel-read:
ParallelRead is the same as baseline with the difference that the file is
split into chunks and each chunk is parsed ino Event objects concurrently

to run, use the following command:
```bash
cargo run -- -n <NUM_THREADS> --trace /path/to/tracefile --mode parallel-read
```

### parallel-parse:
ParallelParse is similar to ParallelRead regarding the reading from file operation.
the difference is that in ParallelRead, each thread reads a Event object from file
and then parse it as for ParallelParse, the reading and parsing operation is seperated
and different threads may be assigned to handle those for an event object.

to run, use the following command:
```bash
cargo run -- -n <NUM_THREADS> --trace /path/to/tracefile --mode parallel-parse
```

### Parallel-cct:
ParallelCCT uses ParallelRead for reading from file and uses parallelism in CCT construction
precedure in which, each CCT is handled by a different thread.

to run, use the following command:
```bash
cargo run -- -n <NUM_THREADS> --trace /path/to/tracefile --mode parallel-cct
```
