pub fn is_valid_language_tag(language_tag: &str, maximum_bytes: usize) -> bool {
    if language_tag.is_empty() || language_tag.len() > maximum_bytes || !language_tag.is_ascii() {
        return false;
    }

    let subtags = language_tag.split('-').collect::<Vec<_>>();
    if subtags.iter().any(|subtag| subtag.is_empty()) {
        return false;
    }

    if subtags[0].eq_ignore_ascii_case("x") {
        return subtags.len() > 1
            && subtags[1..]
                .iter()
                .all(|part| (1..=8).contains(&part.len()) && is_ascii_alphanumeric(part));
    }

    let language = subtags[0];
    let language_length = language.len();
    if !is_ascii_alphabetic(language)
        || !((2..=3).contains(&language_length)
            || language_length == 4
            || (5..=8).contains(&language_length))
    {
        return false;
    }
    let mut position = 1;

    if language_length <= 3 {
        for _ in 0..3 {
            if subtags
                .get(position)
                .is_some_and(|part| part.len() == 3 && is_ascii_alphabetic(part))
            {
                position += 1;
            } else {
                break;
            }
        }
    }

    if subtags
        .get(position)
        .is_some_and(|part| part.len() == 4 && is_ascii_alphabetic(part))
    {
        position += 1;
    }
    if subtags.get(position).is_some_and(|part| {
        (part.len() == 2 && is_ascii_alphabetic(part))
            || (part.len() == 3 && part.bytes().all(|byte| byte.is_ascii_digit()))
    }) {
        position += 1;
    }

    while subtags.get(position).is_some_and(|part| is_variant(part)) {
        position += 1;
    }

    while subtags
        .get(position)
        .is_some_and(|part| is_extension_singleton(part))
    {
        position += 1;
        let extension_start = position;
        while subtags
            .get(position)
            .is_some_and(|part| (2..=8).contains(&part.len()) && is_ascii_alphanumeric(part))
        {
            position += 1;
        }
        if position == extension_start {
            return false;
        }
    }

    if subtags
        .get(position)
        .is_some_and(|part| part.eq_ignore_ascii_case("x"))
    {
        position += 1;
        let private_use_start = position;
        while subtags
            .get(position)
            .is_some_and(|part| (1..=8).contains(&part.len()) && is_ascii_alphanumeric(part))
        {
            position += 1;
        }
        if position == private_use_start {
            return false;
        }
    }

    position == subtags.len()
}

fn is_ascii_alphabetic(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphabetic())
}

fn is_ascii_alphanumeric(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn is_variant(value: &str) -> bool {
    ((5..=8).contains(&value.len()) || (value.len() == 4 && value.as_bytes()[0].is_ascii_digit()))
        && is_ascii_alphanumeric(value)
}

fn is_extension_singleton(value: &str) -> bool {
    value.len() == 1 && is_ascii_alphanumeric(value) && !value.eq_ignore_ascii_case("x")
}

#[cfg(test)]
mod tests {
    use super::is_valid_language_tag;

    #[test]
    fn accepts_common_bcp47_language_tags() {
        for tag in [
            "en",
            "en-GB",
            "zh-Hant",
            "ar",
            "x-peerfora",
            "en-u-ca-gregory",
        ] {
            assert!(is_valid_language_tag(tag, 35), "{tag}");
        }
    }

    #[test]
    fn rejects_malformed_or_overlong_language_tags() {
        for tag in ["", "e", "en_US", "en--GB", "en-", "x", "en-a", "en-12"] {
            assert!(!is_valid_language_tag(tag, 35), "{tag}");
        }
        assert!(!is_valid_language_tag("en-GB", 4));
    }
}
