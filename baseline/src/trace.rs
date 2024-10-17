use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct Trace {
    #[serde(rename = "traceEvents")]
    pub events: Vec<Event>,
}

pub type ProcessId = i32;
pub type ThreadId = i32;
pub type Scope = String;
pub type Id = usize;
pub type Category = String;

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Default)]
pub struct Event {
    pub name: String,

    #[serde(rename = "cat")]
    pub category: Category,
    #[serde(default)]
    #[serde(deserialize_with = "utils::de_hex_to_int")]
    pub id: Id,
    #[serde(default)]
    pub scope: Scope,

    #[serde(rename = "ph")]
    pub phase_type: EventPhase,

    pub pid: ProcessId,
    pub tid: ThreadId,

    #[serde(rename = "ts")]
    pub timestamp: i64,
    #[serde(rename = "dur")]
    pub duration: Option<i64>,

    pub args: Option<serde_json::Value>,
}

impl Event {
    pub fn merge(&mut self, other: &mut Self) {
        if !other.name.is_empty() {
            std::mem::swap(&mut self.name, &mut other.name);
            //self.name = other.name
        }
        if !other.category.is_empty() {
            std::mem::swap(&mut self.category, &mut other.category);
        }
        if other.id != 0 {
            self.id = other.id
        }
        if !other.scope.is_empty() {
            std::mem::swap(&mut self.scope, &mut other.scope);
        }
        if other.pid != 0 {
            self.pid = other.pid
        }
        if other.tid != 0 {
            self.tid = other.tid
        }
        if other.args.is_some() {
            std::mem::swap(&mut self.args, &mut other.args);
        }
    }
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

impl std::fmt::Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}]:{}<s: {},id: {},cat: {}| pid: {},tid: {}>",
            self.phase_type, self.timestamp, self.scope, self.id, self.category, self.pid, self.tid
        )
    }
}

#[derive(Debug, Deserialize, Clone, Eq, PartialEq, Hash, Default)]
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
    #[default]
    Sample,
    #[serde(rename = "X")]
    Complete,
    #[serde(rename = "C")]
    Counter,
}
impl std::fmt::Display for EventPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let display = match self {
            EventPhase::SyncBegin => "B",
            EventPhase::SyncEnd => "E",
            EventPhase::SyncInstant => "i",
            EventPhase::AsyncBegin => "b",
            EventPhase::AsyncEnd => "e",
            EventPhase::AsyncInstant => "n",
            EventPhase::FlowStart => "s",
            EventPhase::FlowEnd => "f",
            EventPhase::FlowStep => "t",
            EventPhase::ObjectCreate => "N",
            EventPhase::ObjectDestroy => "D",
            EventPhase::ObjectSnapshot => "O",
            EventPhase::MemoryDumpGlobal => "V",
            EventPhase::MemoryDumpProcess => "v",
            EventPhase::ContextEnter => "(",
            EventPhase::ContextLeave => ")",
            EventPhase::Metadata => "M",
            EventPhase::Mark => "R",
            EventPhase::Clock => "c",
            EventPhase::Sample => "P",
            EventPhase::Complete => "X",
            EventPhase::Counter => "C",
        };
        write!(f, "{display}")
    }
}
