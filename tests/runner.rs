
use turtlers::turtle_state::*;
use turtlers::turtle::*;
use anyhow::Result;

pub struct Runner {
    pub turtle: Turtle
}



impl Runner {
    /// if load_state=true, the state is immediately loaded to WorldState. Otherwise it will only
    /// be used when executing commands, so that it will eventually be revealed.
    pub fn new(state_name: &str, load_state: bool) -> Self {

        let turtle = Turtle::new("test".to_string());

        let mut runner = Runner {
            turtle
        };

        if load_state {
            runner.load_state(state_name).unwrap();
        }
        runner
    }

    fn load_state(&mut self, id: &str) -> Result<()>{
        let state = deserialize_worldstate(id)?;
        self.turtle.state.world.update_all(state);
        Ok(())
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn create_runner_test() {
        let runner =  Runner::new("0", true);
        println!("{}", runner.turtle.state.world.state.len())

    }
}