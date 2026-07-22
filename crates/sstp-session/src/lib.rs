#![forbid(unsafe_code)]

mod state;

pub use state::{
    Session, SessionAction, SessionCommand, SessionError, SessionPhase, SessionTransition,
};
