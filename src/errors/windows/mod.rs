use crate::errors::AutoGuiError;

impl From<windows_result::Error> for AutoGuiError {
    fn from(err: windows_result::Error) -> Self {
        AutoGuiError::OSFailure(err.message())
    }
}
