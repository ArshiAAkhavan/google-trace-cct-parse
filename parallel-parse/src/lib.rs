use std::io::Result;
use std::path::Path;

mod read;
use baseline::ApplicationCCT;
use baseline::Trace;
use read::parallel_parse;

/// collect_traces reads a tracefile and construct a Trace
pub fn collect_traces(trace_path: &Path) -> Result<Trace> {
    parallel_parse(trace_path)
}

/// build_application_cct reads the Trace and creates the ApplicationCCT
/// from the trace
pub fn build_application_cct(trace: Trace) -> ApplicationCCT {
    baseline::build_application_cct(trace)
}
