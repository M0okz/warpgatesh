pub const SSH_INCLUDE_LINE: &str = "Include ~/.ssh/warpgatesh/config";

#[must_use]
pub fn ensure_managed_include(existing: &str) -> (String, bool) {
    if existing.lines().any(|line| line.trim() == SSH_INCLUDE_LINE) {
        return (existing.to_owned(), false);
    }

    if existing.is_empty() {
        return (format!("{SSH_INCLUDE_LINE}\n"), true);
    }

    (format!("{SSH_INCLUDE_LINE}\n\n{existing}"), true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn puts_the_include_before_existing_configuration() {
        let (updated, changed) = ensure_managed_include("Host example\n  User gregory\n");
        assert!(changed);
        assert_eq!(
            updated,
            "Include ~/.ssh/warpgatesh/config\n\nHost example\n  User gregory\n"
        );
    }

    #[test]
    fn is_idempotent() {
        let existing = "Include ~/.ssh/warpgatesh/config\n\nHost example\n";
        let (updated, changed) = ensure_managed_include(existing);
        assert!(!changed);
        assert_eq!(updated, existing);
    }
}
