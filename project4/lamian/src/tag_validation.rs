pub const EMPTY_SEGMENT_REASON: &str = "tag hierarchy segments cannot be empty";
pub const INVALID_CHARACTER_REASON: &str =
    "tag can only include letters, numbers, underscore, hyphen, and colon";

#[derive(Debug, Clone)]
pub enum TagValidationError {
    MissingTag,
    InvalidTag { reason: &'static str, value: String },
}

pub fn normalize_and_validate_tag(raw_tag: &str) -> Result<String, TagValidationError> {
    let normalized_tag = raw_tag.trim().to_ascii_lowercase();
    if normalized_tag.is_empty() {
        return Err(TagValidationError::MissingTag);
    }

    if normalized_tag.starts_with(':')
        || normalized_tag.ends_with(':')
        || normalized_tag.contains("::")
    {
        return Err(TagValidationError::InvalidTag {
            reason: EMPTY_SEGMENT_REASON,
            value: normalized_tag,
        });
    }

    for segment in normalized_tag.split(':') {
        if segment.is_empty() {
            return Err(TagValidationError::InvalidTag {
                reason: EMPTY_SEGMENT_REASON,
                value: normalized_tag,
            });
        }

        if !segment.chars().all(is_valid_tag_character) {
            return Err(TagValidationError::InvalidTag {
                reason: INVALID_CHARACTER_REASON,
                value: normalized_tag,
            });
        }
    }

    Ok(normalized_tag)
}

fn is_valid_tag_character(value: char) -> bool {
    value.is_ascii_lowercase() || value.is_ascii_digit() || value == '_' || value == '-'
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_and_validate_tag, TagValidationError, EMPTY_SEGMENT_REASON,
        INVALID_CHARACTER_REASON,
    };

    #[test]
    fn normalize_and_validate_tag_normalizes_case_and_whitespace() {
        let normalized_tag =
            normalize_and_validate_tag("  JWST:Machine_Learning  ").expect("normalize valid tag");
        assert_eq!(normalized_tag, "jwst:machine_learning");
    }

    #[test]
    fn normalize_and_validate_tag_rejects_missing_tag() {
        let error = normalize_and_validate_tag(" \t ").expect_err("empty tag should fail");
        assert!(matches!(error, TagValidationError::MissingTag));
    }

    #[test]
    fn normalize_and_validate_tag_rejects_empty_segments() {
        let error = normalize_and_validate_tag("jwst::ml").expect_err("empty segment should fail");
        let TagValidationError::InvalidTag { reason, value } = error else {
            panic!("unexpected error variant for empty segment");
        };
        assert_eq!(reason, EMPTY_SEGMENT_REASON);
        assert_eq!(value, "jwst::ml");
    }

    #[test]
    fn normalize_and_validate_tag_rejects_invalid_characters() {
        let error =
            normalize_and_validate_tag("jwst:ml!").expect_err("invalid character should fail");
        let TagValidationError::InvalidTag { reason, value } = error else {
            panic!("unexpected error variant for invalid character");
        };
        assert_eq!(reason, INVALID_CHARACTER_REASON);
        assert_eq!(value, "jwst:ml!");
    }
}
