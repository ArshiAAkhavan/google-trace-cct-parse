use cct::build_application_cct;
use std::{collections::HashMap, error::Error, fs::File, io::BufReader};

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    //return Ok(());
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
    dbg!(type_count);
    let app_cct = build_application_cct(trace);
    //let cct = app_cct.sync_tasks.get_mut(&(134511, 5)).unwrap();
    //let roots = cct::build_visual_tree(cct.normalize());
    //println!("{cct}");
    //cct::visualize_tree(&roots);
    let mut x: Vec<((i32, i32), cct::CCT)> = app_cct
        .sync_tasks
        .into_iter()
        .map(|(k, v)| (k, v))
        .collect();
    x.sort_by_key(|x| x.0);
    for (id, cct) in x.iter_mut() {
        println!("sync: {id:#?}");
        println!("{cct}");
        let roots = cct::build_visual_tree(&cct.normalize());
        cct::visualize_tree(&roots, 160);
    }

    //for (id, cct) in app_cct.sync_tasks {
    //    println!("sync: {id:#?}");
    //    println!("{cct}");
    //}
    //
    //for (id, cct) in app_cct.async_tasks {
    //    println!("async: {id:#?}");
    //    println!("{cct}");
    //}
    //
    //for (id, cct) in app_cct.object_life_cycle {
    //    println!("object: {id:#?}");
    //    println!("{cct}");
    //}

    Ok(())
}
