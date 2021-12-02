use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug)]
pub struct SubscriberName(String);

impl SubscriberName {
    pub fn parse(s: String) -> Result<SubscriberName, String> {
        let is_empty_string_or_whitespace = s.trim().is_empty();
        let is_too_long = s.graphemes(true).count() > 256;
    
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_characters = s.chars().any(|char| forbidden_characters.contains(&char));
    
        let is_valid_name = !(is_empty_string_or_whitespace || is_too_long || contains_forbidden_characters);

        if is_valid_name {
            Ok(Self(s))
        } else {
            Err(format!("{} is not a valid subscriber name", s))
        }
    }

    pub fn inner(self) -> String {
        self.0
    }

    pub fn inner_mut(&mut self) -> &mut str {
        &mut self.0
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberName;
    use claim::{assert_err, assert_ok};

    #[test]
    fn a_256_grapheme_name_is_valid() {
        let name = "a".repeat(256);
        assert_ok!(SubscriberName::parse(name));
    }

    #[test]
    fn a_name_longer_than_256_graphemes_is_not_valid() {
        let name = "a".repeat(257);
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn rejects_whitespace_names() {
        let name = "  ".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn rejects_empty_names() {
        let name = "".to_string();
        assert_err!(SubscriberName::parse(name));
    }

    #[test]
    fn names_with_invalid_characters_are_rejected() {
        let forbidden_characters = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];

        for name in forbidden_characters {
            let name = name.to_string();
            assert_err!(SubscriberName::parse(name));
        }
    }

    #[test]
    fn valid_names_are_accepted() {
        let name = "Tony Hawk".to_string();
        assert_ok!(SubscriberName::parse(name));
    }
}