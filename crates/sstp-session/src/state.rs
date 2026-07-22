use core::fmt;
use sstp_protocol::SstpVersion;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum SessionPhase {
    #[default]
    Idle,
    Connecting,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionCommand {
    Connect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionAction {
    OpenOuterTransport { version: SstpVersion },
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct Session {
    phase: SessionPhase,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionTransition {
    session: Session,
    action: SessionAction,
}

impl SessionTransition {
    #[must_use]
    pub const fn session(self) -> Session {
        self.session
    }

    #[must_use]
    pub const fn action(self) -> SessionAction {
        self.action
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SessionError {
    AlreadyStarted(SessionPhase),
}

impl fmt::Display for SessionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::AlreadyStarted(phase) => {
                write!(
                    formatter,
                    "接続処理は現在の段階から開始できません: {phase:?}"
                )
            }
        }
    }
}

impl std::error::Error for SessionError {}

impl Session {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            phase: SessionPhase::Idle,
        }
    }

    #[must_use]
    pub const fn phase(self) -> SessionPhase {
        self.phase
    }

    pub fn handle(self, command: SessionCommand) -> Result<SessionTransition, SessionError> {
        match (self.phase, command) {
            (SessionPhase::Idle, SessionCommand::Connect) => Ok(SessionTransition {
                session: Self {
                    phase: SessionPhase::Connecting,
                },
                action: SessionAction::OpenOuterTransport {
                    version: SstpVersion::current(),
                },
            }),
            (phase, SessionCommand::Connect) => Err(SessionError::AlreadyStarted(phase)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Session, SessionAction, SessionCommand, SessionError, SessionPhase};
    use sstp_protocol::SstpVersion;

    #[test]
    fn connect_moves_idle_session_to_connecting() -> Result<(), SessionError> {
        let transition = Session::new().handle(SessionCommand::Connect)?;

        assert_eq!(transition.session().phase(), SessionPhase::Connecting);
        assert_eq!(
            transition.action(),
            SessionAction::OpenOuterTransport {
                version: SstpVersion::V1_0,
            }
        );
        Ok(())
    }

    #[test]
    fn connect_is_rejected_after_session_started() -> Result<(), SessionError> {
        let connecting = Session::new().handle(SessionCommand::Connect)?.session();

        assert_eq!(
            connecting.handle(SessionCommand::Connect),
            Err(SessionError::AlreadyStarted(SessionPhase::Connecting))
        );
        Ok(())
    }
}
