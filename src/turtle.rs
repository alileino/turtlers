use crate::turtle_program::*;
use crate::turtle_action::*;
use crate::turtle_state::*;
use crate::run_history::*;
use anyhow::Result;

pub struct Turtle {
    pub id: String,
    pub program: Box<dyn TurtleProgram>,
    pub last_action: Option<TurtleAction>,
    pub state: TurtleState,
    pub run_history: RunHistory
}

impl Turtle {
    pub fn new(name: String, ser_policy: StateSerializationPolicy) -> Self {

        let run_history =  RunHistory::new(name.clone());
        let state = TurtleState::new(name.clone(), ser_policy);
        run_history.add_initial_state(&state);
        Turtle {
            id: name.clone(),
            program: Box::new(NoProgram{}),
            last_action: None,
            state,
            run_history
        }
    }

    pub fn from(name: String, state: TurtleState) -> Self {

        let run_history =  RunHistory::new(name.clone());
        run_history.add_initial_state(&state);
        Turtle {
            id: name.clone(),
            program: Box::new(NoProgram{}),
            last_action: None,
            state,
            run_history
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
        self.run_history.add_action(&action);
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