use core::fmt;
use sstp_session::{Session, SessionAction, SessionCommand, SessionError};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HarnessCommand {
    PlanConnect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ConnectPlan {
    pub action: SessionAction,
}

#[derive(Debug)]
pub enum HarnessError {
    MissingCommand,
    UnknownCommand(String),
    UnexpectedArgument(String),
    Session(SessionError),
}

impl fmt::Display for HarnessError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => formatter.write_str("使用方法: sstp-harness plan-connect"),
            Self::UnknownCommand(command) => write!(formatter, "不明なコマンドです: {command}"),
            Self::UnexpectedArgument(argument) => {
                write!(formatter, "予期しない引数です: {argument}")
            }
            Self::Session(source) => write!(formatter, "接続状態の遷移に失敗しました: {source}"),
        }
    }
}

impl std::error::Error for HarnessError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Session(source) => Some(source),
            Self::MissingCommand | Self::UnknownCommand(_) | Self::UnexpectedArgument(_) => None,
        }
    }
}

impl From<SessionError> for HarnessError {
    fn from(source: SessionError) -> Self {
        Self::Session(source)
    }
}

pub fn parse(mut arguments: impl Iterator<Item = String>) -> Result<HarnessCommand, HarnessError> {
    let command = arguments.next().ok_or(HarnessError::MissingCommand)?;
    if let Some(argument) = arguments.next() {
        return Err(HarnessError::UnexpectedArgument(argument));
    }

    match command.as_str() {
        "plan-connect" => Ok(HarnessCommand::PlanConnect),
        _ => Err(HarnessError::UnknownCommand(command)),
    }
}

pub fn execute(command: HarnessCommand) -> Result<ConnectPlan, HarnessError> {
    match command {
        HarnessCommand::PlanConnect => {
            let transition = Session::new().handle(SessionCommand::Connect)?;
            Ok(ConnectPlan {
                action: transition.action(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{HarnessCommand, HarnessError, execute, parse};
    use sstp_protocol::SstpVersion;
    use sstp_session::SessionAction;

    #[test]
    fn parser_accepts_plan_connect() -> Result<(), HarnessError> {
        let arguments = [String::from("plan-connect")].into_iter();

        assert_eq!(parse(arguments)?, HarnessCommand::PlanConnect);
        Ok(())
    }

    #[test]
    fn plan_connect_exposes_only_a_typed_action() -> Result<(), HarnessError> {
        let plan = execute(HarnessCommand::PlanConnect)?;

        assert_eq!(
            plan.action,
            SessionAction::OpenOuterTransport {
                version: SstpVersion::V1_0,
            }
        );
        Ok(())
    }
}
