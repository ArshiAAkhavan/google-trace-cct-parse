use baseline::{Event, Trace};
use rayon::prelude::*;
use std::{
    fs::File,
    io::{BufRead, BufReader, Result, Seek, SeekFrom},
    path::Path,
};

use log::{debug, warn};

type Line = Vec<u8>;

fn read_chunk(file: File, start_pos: u64, chunk_size: usize) -> Result<Vec<Line>> {
    let mut lines = Vec::new();
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
            lines.push(buf[..nbytes].to_vec());
            buf.clear();
        } else {
            break;
        }
    }
    Ok(lines)
}

fn collect_lines(
    thread_id: usize,
    file_path: &Path,
    chunk_size: usize,
    init_skip: u64,
) -> std::io::Result<Vec<Line>> {
    let file = File::open(file_path)?;
    let start_pos = init_skip + (thread_id * chunk_size) as u64;
    let lines = read_chunk(file, start_pos, chunk_size)?;
    Ok(lines)
}

pub fn parallel_parse(file_path: &Path) -> Result<Trace> {
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
        .filter_map(move |thread_id| {
            collect_lines(*thread_id, file_path, chunk_size, init_skip as u64).ok()
        })
        .flatten()
        .filter_map(|line| serde_json::from_slice::<Event>(&line).ok())
        .collect();

    Ok(Trace { events })
}

#[cfg(test)]
mod test {
    use std::path::Path;

    #[test]
    fn check_events_are_parsed_correctly() -> std::io::Result<()> {
        let file_path = "../data/trace-valid-ending.json";
        let trace_sync = baseline::collect_traces(Path::new(file_path))?;
        let trace_parallel = super::parallel_parse(Path::new(file_path))?;

        assert_eq!(trace_sync.events.len(), trace_parallel.events.len());

        for (event1, event2) in trace_sync.events.iter().zip(trace_parallel.events.iter()) {
            assert_eq!(event1, event2);
        }

        Ok(())
    }
}
