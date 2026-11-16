use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyStepError {
    #[error("a Wait step cannot sit inside SameFrame: a frame group holds key events only")]
    WaitInSameFrame,
}

#[cfg(test)]
mod tests {
    use super::KeyStepError;

    #[test]
    fn test_usage() {
        let error = KeyStepError::WaitInSameFrame;
        assert_eq!(
            error.to_string(),
            "a Wait step cannot sit inside SameFrame: a frame group holds key events only"
        );
    }
}
