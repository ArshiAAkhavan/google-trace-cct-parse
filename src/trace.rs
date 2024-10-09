use std::collections::HashMap;

use crate::CCT;
use serde::Deserialize;

type ProcessId = i32;
type ThreadId = i32;
type SyncTaskId = (ProcessId, ThreadId);

type Scope = String;
type Id = usize;
type Category = String;
type AsyncTaskId = (Scope, Id, Category);

#[derive(Debug, Default)]
pub struct ApplicationTrace {
    pub sync_tasks: HashMap<SyncTaskId, CCT>,
    pub async_tasks: HashMap<AsyncTaskId, CCT>,
}

impl ApplicationTrace {
    pub fn new() -> Self {
        Default::default()
    }
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
    pub category: Category,
    #[serde(default)]
    #[serde(deserialize_with = "crate::utils::de_hex_to_int")]
    pub id: Id,
    #[serde(default)]
    pub scope: Scope,

    #[serde(rename = "ph")]
    pub phase_type: EventPhase,
    pub pid: ProcessId,
    pub tid: ThreadId,
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
