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

pub enum ProgramState {
    Finished,
    _Waiting(f64), // waiting for turtle to report back
    HasInstructions(f64) // has instructions that can be delivered to turtle
}

impl Turtle {
    pub fn new(name: String) -> Self {
        Turtle {
            id: name.clone(),
            program: Box::new(NoProgram{}),
            last_action: None,
            state: TurtleState::new(name)
        }
    }

    fn program_state(&self) -> ProgramState {
        let progress = self.program.progress();
        if progress.0 == progress.1 {
            return ProgramState::Finished;
        }
        let progress_f = (progress.1 as f64)/(progress.0 as f64);
        ProgramState::HasInstructions(progress_f)
    }

    pub fn set_program(&mut self, program: Box<dyn TurtleProgram>) {
        self.program = program;
        println!("Set program to {}", self.program.name());
    }

    fn record(&mut self, action: TurtleAction) {
        println!("{:?}", action);
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