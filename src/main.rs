use cct::{build_application_trace, EventPhase, Trace};
use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    //let data = File::open("data/trace-1.json")?;
    //let data = File::open("data/trace-heavy.json")?;
    //let data = File::open("data/sample.json")?;
    let data = File::open("data/trace-valid-ending.json")?;
    let data = BufReader::new(data);
    let trace: cct::Trace = serde_json::from_reader(data)?;
    dbg!(trace.events.len());
    let mut type_count = HashMap::new();

    for event in trace.events.iter() {
        type_count
            .entry(&event.phase_type)
            .and_modify(|x| *x += 1)
            .or_insert(1);
    }

    for event in trace.events.iter() {
        match event.phase_type {
            EventPhase::AsyncBegin | EventPhase::AsyncEnd | EventPhase::AsyncInstant => {
                println!("{event:#?}")
            }
            _ => (),
        }
    }
    dbg!(type_count);
    build_application_trace(trace);

    Ok(())
}
