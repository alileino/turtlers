use crate::turtle_program::*;
use crate::turtle_action::*;
use crate::turtle_state::*;
use anyhow::Result;

pub struct Turtle {
    pub id: String,
    pub program: Box<dyn TurtleProgram>,
    pub last_action: Option<TurtleAction>,
    pub state: TurtleState
}



impl Turtle {
    pub fn new(name: String, ser_policy: StateSerializationPolicy) -> Self {
        Turtle {
            id: name.clone(),
            program: Box::new(NoProgram{}),
            last_action: None,
            state: TurtleState::new(name, ser_policy)
        }
    }

    pub fn from(name: String, state: TurtleState) -> Self {
        Turtle {
            id: name.clone(),
            program: Box::new(NoProgram{}),
            last_action: None,
            state
        }
    }

    fn program_state(&self) -> ProgramState {
        program_state(&self.program)
    }

    pub fn set_program(&mut self, program: Box<dyn TurtleProgram>) {
        self.program = program;
        println!("Set program to {}", self.program.name());
    }

    fn record(&mut self, action: TurtleAction) {
        self.last_action = Some(action);
    }

    pub fn update(&mut self, result: &TurtleActionReturn) {
        let action = self.last_action.as_ref().unwrap();
        self.state.update(action, result);
        self.program.update(&self.state, action, result);
    }


    pub fn next(&mut self) -> Result<&TurtleAction> {
        let action = match self.program_state() {
            ProgramState::HasInstructions(_) => self.program.next()?,
            ProgramState::Finished => TurtleAction::Stop,
            _ => panic!()
        };
        self.record(action);
        Ok(&self.last_action.as_ref().unwrap())
    }
}