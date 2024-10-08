use cct::{build_application_trace, Trace};
use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Hello, world!");
    //let data = File::open("data/trace-1.json")?;
    let data = File::open("data/trace-heavy.json")?;
    //let data = File::open("data/sample.json")?;
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
    dbg!(type_count);
    build_application_trace(trace);

    Ok(())
}
