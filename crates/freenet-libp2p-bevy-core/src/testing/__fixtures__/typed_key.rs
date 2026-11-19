use bevy::input::keyboard::{Key, KeyCode, NativeKeyCode};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedKey {
    pub key_code: KeyCode,
    pub logical_key: Key,
    pub needs_shift: bool,
}

impl From<char> for TypedKey {
    fn from(character: char) -> Self {
        let (key_code, needs_shift) = match character {
            'a' | 'A' => (KeyCode::KeyA, character.is_ascii_uppercase()),
            'b' | 'B' => (KeyCode::KeyB, character.is_ascii_uppercase()),
            'c' | 'C' => (KeyCode::KeyC, character.is_ascii_uppercase()),
            'd' | 'D' => (KeyCode::KeyD, character.is_ascii_uppercase()),
            'e' | 'E' => (KeyCode::KeyE, character.is_ascii_uppercase()),
            'f' | 'F' => (KeyCode::KeyF, character.is_ascii_uppercase()),
            'g' | 'G' => (KeyCode::KeyG, character.is_ascii_uppercase()),
            'h' | 'H' => (KeyCode::KeyH, character.is_ascii_uppercase()),
            'i' | 'I' => (KeyCode::KeyI, character.is_ascii_uppercase()),
            'j' | 'J' => (KeyCode::KeyJ, character.is_ascii_uppercase()),
            'k' | 'K' => (KeyCode::KeyK, character.is_ascii_uppercase()),
            'l' | 'L' => (KeyCode::KeyL, character.is_ascii_uppercase()),
            'm' | 'M' => (KeyCode::KeyM, character.is_ascii_uppercase()),
            'n' | 'N' => (KeyCode::KeyN, character.is_ascii_uppercase()),
            'o' | 'O' => (KeyCode::KeyO, character.is_ascii_uppercase()),
            'p' | 'P' => (KeyCode::KeyP, character.is_ascii_uppercase()),
            'q' | 'Q' => (KeyCode::KeyQ, character.is_ascii_uppercase()),
            'r' | 'R' => (KeyCode::KeyR, character.is_ascii_uppercase()),
            's' | 'S' => (KeyCode::KeyS, character.is_ascii_uppercase()),
            't' | 'T' => (KeyCode::KeyT, character.is_ascii_uppercase()),
            'u' | 'U' => (KeyCode::KeyU, character.is_ascii_uppercase()),
            'v' | 'V' => (KeyCode::KeyV, character.is_ascii_uppercase()),
            'w' | 'W' => (KeyCode::KeyW, character.is_ascii_uppercase()),
            'x' | 'X' => (KeyCode::KeyX, character.is_ascii_uppercase()),
            'y' | 'Y' => (KeyCode::KeyY, character.is_ascii_uppercase()),
            'z' | 'Z' => (KeyCode::KeyZ, character.is_ascii_uppercase()),
            '0' | ')' => (KeyCode::Digit0, character == ')'),
            '1' | '!' => (KeyCode::Digit1, character == '!'),
            '2' | '@' => (KeyCode::Digit2, character == '@'),
            '3' | '#' => (KeyCode::Digit3, character == '#'),
            '4' | '$' => (KeyCode::Digit4, character == '$'),
            '5' | '%' => (KeyCode::Digit5, character == '%'),
            '6' | '^' => (KeyCode::Digit6, character == '^'),
            '7' | '&' => (KeyCode::Digit7, character == '&'),
            '8' | '*' => (KeyCode::Digit8, character == '*'),
            '9' | '(' => (KeyCode::Digit9, character == '('),
            ' ' => (KeyCode::Space, false),
            '-' | '_' => (KeyCode::Minus, character == '_'),
            '=' | '+' => (KeyCode::Equal, character == '+'),
            '[' | '{' => (KeyCode::BracketLeft, character == '{'),
            ']' | '}' => (KeyCode::BracketRight, character == '}'),
            '\\' | '|' => (KeyCode::Backslash, character == '|'),
            ';' | ':' => (KeyCode::Semicolon, character == ':'),
            '\'' | '"' => (KeyCode::Quote, character == '"'),
            ',' | '<' => (KeyCode::Comma, character == '<'),
            '.' | '>' => (KeyCode::Period, character == '>'),
            '/' | '?' => (KeyCode::Slash, character == '?'),
            '`' | '~' => (KeyCode::Backquote, character == '~'),
            _ => (KeyCode::Unidentified(NativeKeyCode::Unidentified), false),
        };
        let logical_key = if character == ' ' {
            Key::Space
        } else {
            Key::Character(character.to_string().into())
        };
        Self {
            key_code,
            logical_key,
            needs_shift,
        }
    }
}

#[cfg(test)]
mod tests {
    use bevy::input::keyboard::{Key, KeyCode, NativeKeyCode};

    use super::TypedKey;

    #[test]
    fn test_usage() {
        assert_eq!(
            TypedKey::from('a'),
            TypedKey {
                key_code: KeyCode::KeyA,
                logical_key: Key::Character("a".into()),
                needs_shift: false,
            }
        );
        assert_eq!(
            TypedKey::from('B'),
            TypedKey {
                key_code: KeyCode::KeyB,
                logical_key: Key::Character("B".into()),
                needs_shift: true,
            }
        );
        assert_eq!(TypedKey::from('1').key_code, KeyCode::Digit1);
        assert_eq!(TypedKey::from('!').key_code, KeyCode::Digit1);
        assert!(TypedKey::from('!').needs_shift);
        assert_eq!(
            TypedKey::from(' '),
            TypedKey {
                key_code: KeyCode::Space,
                logical_key: Key::Space,
                needs_shift: false,
            }
        );
        assert_eq!(TypedKey::from('-').key_code, KeyCode::Minus);
        assert_eq!(TypedKey::from('_').key_code, KeyCode::Minus);
        assert!(TypedKey::from('_').needs_shift);
        assert_eq!(
            TypedKey::from('é').key_code,
            KeyCode::Unidentified(NativeKeyCode::Unidentified)
        );
        assert!(!TypedKey::from('é').needs_shift);
    }
}
