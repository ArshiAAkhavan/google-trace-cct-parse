use log::info;
use std::path::PathBuf;

use clap::{Parser, ValueEnum};

#[derive(Clone, ValueEnum, Default, PartialEq)]
pub enum Mode {
    /// Baseline is the sequential mode in which data is read sequentially, parsed
    /// sequentially and calling context trees are formed sequentially.
    #[default]
    Baseline,

    /// ParallelRead is the same as baseline with the difference that the file is
    /// split into chunks and each chunk is parsed ino Event objects concurrently
    ParallelRead,

    /// ParallelParse is similar to ParallelRead regarding the reading from file operation.
    /// the difference is that in ParallelRead, each thread reads a Event object from file
    /// and then parse it as for ParallelParse, the reading and parsing operation is seperated
    /// and different threads may be assigned to handle those for an event object.
    ParallelParse,

    /// ParallelCCT uses ParallelRead for reading from file and uses parallelism in CCT construction
    /// precedure in which, each CCT is handled by a different thread.
    ParallelCCT,
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(propagate_version = true)]
#[clap(rename_all = "kebab_case")]
struct Opts {
    /// Address of the trace file
    #[arg(short, long)]
    trace: PathBuf,

    /// which implementation to run
    #[arg(short, long)]
    mode: Mode,
}

fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let opts = Opts::parse();

    info!("trace file: {}", opts.trace.to_string_lossy());
    match opts.mode {
        Mode::Baseline => run_baseline(opts.trace),
        Mode::ParallelRead => run_parallel_read(opts.trace),
        Mode::ParallelParse => run_parallel_parse(opts.trace),
        Mode::ParallelCCT => run_parallel_cct(opts.trace),
    }
}

fn consume<T>(data: T) {
    _ = data;
}

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

        println!("{}: {}", stringify!($func), duration.as_millis());
        result
    }};
}

macro_rules! gen_bench {
    ($crate_name:ident) => {
        paste::item! {
            fn [<run_ $crate_name>](trace: std::path::PathBuf) -> std::io::Result<()> {
                use std::path::Path;
                use $crate_name::{build_application_cct, collect_traces};

                let trace = track!(collect_traces(Path::new(&trace)))?;
                let app_cct = track!(build_application_cct(trace));
                consume(app_cct);
                Ok(())
            }
        }
    };
}

gen_bench!(baseline);
gen_bench!(parallel_read);
gen_bench!(parallel_parse);
gen_bench!(parallel_cct);
