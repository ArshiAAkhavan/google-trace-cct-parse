use std::collections::HashMap;

use baseline::{ApplicationCCT, Category, Event, Id, ProcessId, Scope, ThreadId, CCT};
use rayon::iter::{IntoParallelIterator, ParallelIterator};

pub type SyncTaskId = (ProcessId, ThreadId);
type AsyncTaskId = (Scope, Id, Category);
type ObjectLifeCycleId = (Scope, Id);

/// ApplicationTrace is a middle stage that holds a series of vectors of events,
/// each later used to construct a new CCT.
#[derive(Debug, Default)]
pub struct ApplicationTrace {
    pub sync_tasks: HashMap<SyncTaskId, Vec<Event>>,
    pub async_tasks: HashMap<AsyncTaskId, Vec<Event>>,
    pub object_life_cycle: HashMap<ObjectLifeCycleId, Vec<Event>>,
}

impl ApplicationTrace {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn application_cct(self) -> ApplicationCCT {
        let sync_tasks: HashMap<SyncTaskId, CCT> = self
            .sync_tasks
            .into_par_iter()
            .map(|(id, events)| (id, CCT::from(events)))
            .collect();

        let async_tasks: HashMap<AsyncTaskId, CCT> = self
            .async_tasks
            .into_par_iter()
            .map(|(id, events)| (id, CCT::from(events)))
            .collect();

        let object_life_cycle: HashMap<ObjectLifeCycleId, CCT> = self
            .object_life_cycle
            .into_par_iter()
            .map(|(id, events)| (id, CCT::from(events)))
            .collect();

        ApplicationCCT {
            sync_tasks,
            async_tasks,
            object_life_cycle,
        }
    }
}
