use thiserror::Error;

#[derive(Error, Debug)]
pub enum KeyStepError {
    #[error("a Wait step cannot sit inside SameFrame: a frame group holds key events only")]
    WaitInSameFrame,
}
