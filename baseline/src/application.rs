use std::collections::HashMap;

use crate::{Category, Event, Id, ProcessId, Scope, ThreadId, CCT};

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
        let mut app_cct = ApplicationCCT {
            ..Default::default()
        };
        for (task_id, events) in self.sync_tasks {
            app_cct.sync_tasks.insert(task_id, CCT::from(events));
        }
        for (task_id, events) in self.async_tasks {
            app_cct.async_tasks.insert(task_id, CCT::from(events));
        }
        for (object_life_cycle_id, events) in self.object_life_cycle {
            app_cct
                .object_life_cycle
                .insert(object_life_cycle_id, CCT::from(events));
        }
        app_cct
    }
}

/// ApplicationCCT holds the entire CCTs of an application.
/// each CCT is either a sync, async, or an object life cycle CCT.
/// each CCT can be indexed by its unique id.
#[derive(Debug, Default)]
pub struct ApplicationCCT {
    pub sync_tasks: HashMap<SyncTaskId, CCT>,
    pub async_tasks: HashMap<AsyncTaskId, CCT>,
    pub object_life_cycle: HashMap<ObjectLifeCycleId, CCT>,
}
