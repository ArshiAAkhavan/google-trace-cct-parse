use baseline::Event;
use std::{
    fs::File,
    io::{BufRead, BufReader, Result, Seek, SeekFrom},
    path::Path,
};

use log::warn;

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

/// collect_events gets a trace file path and reads a chunk of it to generate events
pub fn collect_events(
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
