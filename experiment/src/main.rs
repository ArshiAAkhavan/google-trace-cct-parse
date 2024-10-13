use baseline::{build_application_cct, collect_traces};
use log::info;

use std::collections::HashSet;

macro_rules! track {
    ($func:expr) => {{
        use std::time::Instant;

        // Start tracking the time
        let start = Instant::now();
        info!("track: [{}]\tstart calculating...", stringify!($func));

        // Capture the function's output
        let result = $func;

        // Calculate the elapsed time
        let duration = start.elapsed();

        // Extract the function name (using stringify for better readability)
        info!(
            "track: [{}]\ttook {} ms to finish",
            stringify!($func),
            duration.as_millis()
        );

        // Return the result of the function
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
        paths
    };
    for trace_path in trace_paths {
        info!("trace file: {trace_path}");
        let trace = track!(collect_traces(trace_path.into()))?;
        let app_cct = track!(build_application_cct(trace));
        consume(app_cct);
    }

    Ok(())
}

fn consume<T>(data: T) {
    _ = data;
}
