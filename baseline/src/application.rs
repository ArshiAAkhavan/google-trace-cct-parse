use std::collections::HashMap;

use crate::{
    trace::{Category, Id, ProcessId, Scope, ThreadId},
    Event, CCT,
};

pub type SyncTaskId = (ProcessId, ThreadId);
type AsyncTaskId = (Scope, Id, Category);
type ObjectLifeCycleId = (Scope, Id);

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
            app_cct.sync_tasks.insert(task_id, CCT::from_events(events));
        }
        for (task_id, events) in self.async_tasks {
            app_cct
                .async_tasks
                .insert(task_id, CCT::from_events(events));
        }
        for (object_life_cycle_id, events) in self.object_life_cycle {
            app_cct
                .object_life_cycle
                .insert(object_life_cycle_id, CCT::from_events(events));
        }
        app_cct
    }
}

#[derive(Debug, Default)]
pub struct ApplicationCCT {
    pub sync_tasks: HashMap<SyncTaskId, CCT>,
    pub async_tasks: HashMap<AsyncTaskId, CCT>,
    pub object_life_cycle: HashMap<ObjectLifeCycleId, CCT>,
}
