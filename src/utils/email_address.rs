pub fn normalize_email_address(email: &str) -> String {
    let trimmed_email = email.trim();
    let Some((local_part, domain)) = trimmed_email.rsplit_once('@') else {
        return trimmed_email.to_owned();
    };
    format!("{local_part}@{}", domain.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::normalize_email_address;

    #[test]
    fn preserves_email_local_part_and_normalizes_domain_case() {
        assert_eq!(
            normalize_email_address("  Student.Name@EXAMPLE.EDU  "),
            "Student.Name@example.edu"
        );
    }

    #[test]
    fn does_not_change_a_non_domain_part_without_at_sign() {
        assert_eq!(normalize_email_address("Student.Name"), "Student.Name");
    }
}
