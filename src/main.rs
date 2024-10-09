use cct::{build_application_trace, EventPhase, Trace};
use std::{any::Any, collections::HashMap, error::Error, fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    //let data = File::open("data/trace-1.json")?;
    //let data = File::open("data/trace-heavy.json")?;
    //let data = File::open("data/sample.json")?;
    let data = File::open("data/trace-valid-ending.json")?;
    let data = BufReader::new(data);
    let trace: cct::Trace = serde_json::from_reader(data)?;
    dbg!(trace.events.len());
    let mut type_count = HashMap::new();
    let mut async_type_count: HashMap<(usize, String, String), Vec<(i32, i32)>> = HashMap::new();

    for event in trace.events.iter() {
        type_count
            .entry(&event.phase_type)
            .and_modify(|x| *x += 1)
            .or_insert(1);

        match event.phase_type {
            EventPhase::AsyncBegin | EventPhase::AsyncEnd | EventPhase::AsyncInstant => {
                async_type_count
                    .entry((event.id, event.scope.clone(), event.category.clone()))
                    .and_modify(|pid_tid| pid_tid.push((event.pid, event.tid)))
                    .or_default();
            }
            _ => (),
        }
    }
    dbg!(type_count);
    'mamad: for (key, ptids) in async_type_count.iter() {
        for (i, ptid) in ptids.iter().enumerate() {
            if i != 0 && *ptid != ptids[i - 1] {
                println!("{key:#?}#{i}: {:#?},{:#?}", ptids[i], ptids[i - 1]);
                break 'mamad;
            }
        }
    }
    build_application_trace(trace);

    Ok(())
}
