use baseline::{Event, Trace};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader, Result, Seek, SeekFrom},
    path::Path,
};

use log::{debug, warn};

/// reads a chunk of the trace file and extract all events from it.
fn read_chunk(file: File, start_pos: u64, chunk_size: usize) -> Result<Vec<Event>> {
    let mut events = Vec::new();
    // Seek to the start position assigned to the thread
    let mut file = file;

    file.seek(SeekFrom::Start(start_pos))?;
    let mut data = BufReader::new(file);

    // denotes weather we reached the end of event array in json file
    let mut end_of_events = false;

    let mut bytes_read = 0;

    let mut buf = Vec::with_capacity(1500);
    while !end_of_events && bytes_read <= chunk_size {
        if let Ok(mut nbytes @ 1..) = data.read_until(b'\n', &mut buf) {
            bytes_read += nbytes;

            // according to the chrome's trace json format, the last line of events, ends with the
            // event object, followed by `]`, followed by the next field in the trace object. so we
            // need to check if the line ends in a valid index-ending, i.e, a `}` followed by a `,`
            // or if it ends with ':' showing that the array is finished and the next field of the
            // trace object is read.
            if !(&buf[nbytes - 2..nbytes] == b",\n") {
                let pattern = b"}],";
                if let Some(pos) = buf
                    .windows(pattern.len())
                    .rposition(|window| window == pattern)
                {
                    // next line substracts nbytes by 2 to address the trailing ",\n"
                    nbytes = pos + 3;
                }
                end_of_events = true;
            }
            nbytes -= 2;

            let event: Event = match serde_json::from_slice(&buf[..nbytes]) {
                Ok(event) => event,
                Err(e) => {
                    warn!(
                        "faced error when parsing {}: {e}",
                        String::from_utf8_lossy(&buf[..nbytes])
                    );
                    buf.clear();
                    continue;
                }
            };
            buf.clear();
            events.push(event);
        } else {
            break;
        }
    }
    Ok(events)
}

/// collect_events gets a trace file path and reads a chunk of it to generate its events
fn collect_events(
    thread_id: usize,
    file_path: &Path,
    chunk_size: usize,
    init_skip: u64,
) -> std::io::Result<Vec<Event>> {
    let file = File::open(file_path)?;
    let start_pos = init_skip + (thread_id * chunk_size) as u64;
    let events: Vec<Event> = read_chunk(file, start_pos, chunk_size)?;
    Ok(events)
}

/// takes a path to the trace file and split the loading and json parsing of it between threads.
/// this function creates a Trace of the given file.
pub fn parallel_read(file_path: &Path) -> Result<Trace> {
    let num_threads = rayon::current_num_threads();
    debug!("concurrency level: {num_threads}");

    let file = File::open(file_path)?;
    let file_size = file.metadata()?.len();

    // ignore the first line since it represents the trace array,
    // i.e., {"traceEvents":[
    let init_skip = BufReader::new(file).read_until(b'\n', &mut vec![]).unwrap();

    let chunk_size = (file_size as usize - init_skip + num_threads - 1) / num_threads;

    let threads: Vec<usize> = (0..num_threads).collect();

    let events = threads
        .par_iter()
        .map(move |thread_id| collect_events(*thread_id, file_path, chunk_size, init_skip as u64))
        .try_reduce(Vec::new, |mut this, mut other| {
            this.append(&mut other);
            Ok(this)
        })?;

    Ok(Trace { events })
}

#[cfg(test)]
mod test {
    use std::path::Path;

    #[test]
    fn check_events_are_read_correctly() -> std::io::Result<()> {
        let file_path = "../data/trace-valid-ending.json";
        let trace_sync = baseline::collect_traces(Path::new(file_path))?;
        let trace_parallel = super::parallel_read(Path::new(file_path))?;

        assert_eq!(trace_sync.events.len(), trace_parallel.events.len());

        for (event1, event2) in trace_sync.events.iter().zip(trace_parallel.events.iter()) {
            assert_eq!(event1, event2);
        }

        Ok(())
    }
}
