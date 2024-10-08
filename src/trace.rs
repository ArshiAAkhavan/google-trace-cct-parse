use std::collections::HashMap;

use serde::Deserialize;

use crate::CallingContextTree;

#[derive(Debug, Deserialize)]
pub struct ApplicationTrace {
    pub processes: HashMap<i32, ProcessTrace>,
}

#[derive(Debug, Deserialize)]
pub struct ProcessTrace {
    pub pid: i32,
    pub threads: HashMap<i32, ThreadTrace>,
}

#[derive(Debug, Deserialize)]
pub struct ThreadTrace {
    pub tid: i32,
    pub cct: CallingContextTree,
}

#[derive(Debug, Deserialize)]
pub struct Trace {
    #[serde(rename = "traceEvents")]
    pub events: Vec<Event>,
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq)]
pub struct Event {
    name: String,

    #[serde(rename = "cat")]
    category: String,
    #[serde(rename = "ph")]
    pub phase_type: EventPhase,
    pub pid: i32,
    pub tid: i32,
    #[serde(rename = "ts")]
    timestamp: i64,
}

impl Ord for Event {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.timestamp.cmp(&other.timestamp)
    }
}

impl PartialOrd for Event {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.timestamp.partial_cmp(&other.timestamp)
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash)]
pub enum EventPhase {
    #[serde(rename = "B")]
    SyncBegin,
    #[serde(rename = "E")]
    SyncEnd,
    #[serde(rename = "i", alias = "I")]
    SyncInstant,

    #[serde(rename = "b")]
    AsyncBegin,
    #[serde(rename = "e")]
    AsyncEnd,
    #[serde(rename = "n")]
    AsyncInstant,

    #[serde(rename = "s")]
    FlowStart,
    #[serde(rename = "f")]
    FlowEnd,
    #[serde(rename = "t")]
    FlowStep,

    #[serde(rename = "N")]
    ObjectCreate,
    #[serde(rename = "D")]
    ObjectDestroy,
    #[serde(rename = "O")]
    ObjectSnapshot,

    #[serde(rename = "V")]
    MemoryDumpGlobal,
    #[serde(rename = "v")]
    MemoryDumpProcess,

    #[serde(rename = "(")]
    ContextEnter,
    #[serde(rename = ")")]
    ContextLeave,

    #[serde(rename = "M")]
    Metadata,
    #[serde(rename = "R")]
    Mark,
    #[serde(rename = "c")]
    Clock,
    #[serde(rename = "P")]
    Sample,
    #[serde(rename = "X")]
    Complete,
    #[serde(rename = "C")]
    Counter,
}
