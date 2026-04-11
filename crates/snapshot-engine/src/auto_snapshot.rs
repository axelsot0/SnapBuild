#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotTrigger {
    OnSave,
    OnRun,
    IdleTimeout,
    Manual,
    OnRestore,
    OnClose,
}

pub fn should_create_snapshot(
    trigger: SnapshotTrigger,
    had_real_edits: bool,
    current_source_fingerprint: &str,
    last_persisted_source_fingerprint: Option<&str>,
) -> bool {
    if current_source_fingerprint.is_empty() {
        return false;
    }

    if Some(current_source_fingerprint) == last_persisted_source_fingerprint {
        return false;
    }

    match trigger {
        SnapshotTrigger::IdleTimeout => had_real_edits,
        SnapshotTrigger::OnSave
        | SnapshotTrigger::OnRun
        | SnapshotTrigger::Manual
        | SnapshotTrigger::OnRestore
        | SnapshotTrigger::OnClose => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skips_when_fingerprint_is_equal_to_last_snapshot() {
        assert!(!should_create_snapshot(
            SnapshotTrigger::OnSave,
            true,
            "abc",
            Some("abc")
        ));
    }

    #[test]
    fn idle_timeout_requires_real_edits() {
        assert!(!should_create_snapshot(
            SnapshotTrigger::IdleTimeout,
            false,
            "abc",
            Some("prev")
        ));
        assert!(should_create_snapshot(
            SnapshotTrigger::IdleTimeout,
            true,
            "abc",
            Some("prev")
        ));
    }
}
