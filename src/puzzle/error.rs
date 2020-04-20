use failure::Fail;

#[derive(Fail, Debug)]
#[fail(display = "Error parsing puzzle: {}", _0)]
pub struct ParsePuzzleError(String);

impl<S: Into<String>> From<S> for ParsePuzzleError {
    fn from(reason: S) -> Self {
        Self(reason.into())
    }
}
