use crate::command::ConnectPlan;
use sstp_session::SessionAction;

#[must_use]
pub fn render(plan: ConnectPlan) -> String {
    match plan.action {
        SessionAction::OpenOuterTransport { version } => {
            format!("SSTP {version} の外側通信路を開く")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::render;
    use crate::command::ConnectPlan;
    use sstp_protocol::SstpVersion;
    use sstp_session::SessionAction;

    #[test]
    fn connect_plan_has_stable_human_readable_output() {
        let plan = ConnectPlan {
            action: SessionAction::OpenOuterTransport {
                version: SstpVersion::V1_0,
            },
        };

        assert_eq!(render(plan), "SSTP 1.0 の外側通信路を開く");
    }
}
