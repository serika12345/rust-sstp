use core::fmt;

/// このクライアントが解釈できるSSTPプロトコル版。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SstpVersion {
    V1_0,
}

impl SstpVersion {
    #[must_use]
    pub const fn current() -> Self {
        Self::V1_0
    }
}

impl fmt::Display for SstpVersion {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::V1_0 => formatter.write_str("1.0"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SstpVersion;

    #[test]
    fn current_version_is_sstp_1_0() {
        assert_eq!(SstpVersion::current(), SstpVersion::V1_0);
    }

    #[test]
    fn version_has_stable_text_representation() {
        assert_eq!(SstpVersion::V1_0.to_string(), "1.0");
    }
}
