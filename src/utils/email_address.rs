pub fn normalize_email_address(email: &str) -> String {
    let trimmed_email = email.trim();
    let Some((local_part, domain)) = trimmed_email.rsplit_once('@') else {
        return trimmed_email.to_owned();
    };
    format!(
        "{}@{}",
        local_part.to_ascii_lowercase(),
        domain.to_ascii_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::normalize_email_address;

    #[test]
    fn normalizes_email_case_and_surrounding_whitespace() {
        assert_eq!(
            normalize_email_address("  Student.Name@EXAMPLE.EDU  "),
            "student.name@example.edu"
        );
    }

    #[test]
    fn does_not_change_a_non_domain_part_without_at_sign() {
        assert_eq!(normalize_email_address("Student.Name"), "Student.Name");
    }
}
