//use baseline::{build_application_cct, collect_traces};
//use parallel_parse::{build_application_cct, collect_traces};
use parallel_read::{build_application_cct, collect_traces};

use log::info;
use std::{collections::HashSet, path::Path};

macro_rules! track {
    ($func:expr) => {{
        use std::time::Instant;

        let start = Instant::now();
        info!("track: [{}]\tstart calculating...", stringify!($func));
        let result = $func;
        let duration = start.elapsed();

        info!(
            "track: [{}]\ttook {} ms to finish",
            stringify!($func),
            duration.as_millis()
        );

        result
    }};
}

fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let trace_paths = {
        let mut paths = HashSet::new();
        paths.insert("data/trace-1.json");
        paths.insert("data/trace-heavy.json");
        paths.insert("data/trace-valid-ending.json");
        paths.insert("data/trace-100%.json");
        paths
    };
    for trace_path in trace_paths {
        info!("trace file: {trace_path}");
        let trace = track!(collect_traces(Path::new(trace_path)))?;
        let app_cct = track!(build_application_cct(trace));
        consume(app_cct);
    }

    Ok(())
}

fn consume<T>(data: T) {
    _ = data;
}
