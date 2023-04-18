use std::time::{SystemTime, UNIX_EPOCH};
use crate::turtle_state::{StateSerializationPolicy, WorldState, TurtleState};
use crate::turtle_action::{TurtleAction, TurtleActionReturn};
use serde::{Serialize, Serializer};

pub fn create_run_id(id: &str) -> String {
    let now = SystemTime::now();
    let duration = now.duration_since(UNIX_EPOCH).unwrap();
    format!("{}_{}", id, duration.as_millis())
}

pub struct RunHistory {
    run_id: String,
    path: String
}

/*
Things to record:
Initial state of world
Initial location
Initial inventory

Action that was carried out
Result of the action

Change of world in result of the action
Change of location in result of the action

 */


impl RunHistory {
    pub fn new(id: String) -> Self {
        let run_id = create_run_id(id.as_str());
        std::fs::create_dir_all("runs").unwrap();

        RunHistory {
            run_id: run_id.clone(),
            path: format!("runs/{}.txt", run_id)
        }
    }

    pub fn add_initial_state(&self, state: &TurtleState) {

    }

    pub fn add_action(&self, action: &TurtleAction) {
        println!("TODO");
        //let action_str = serde_json::to_string(&action).unwrap();
        //println!("SER WORKS: {}", action_str);
    }


}



