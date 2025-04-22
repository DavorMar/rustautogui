use std::{
    ffi::{CString, NulError},
    fmt,
};

#[cfg(feature = "opencl")]
use ocl;

#[derive(Debug)]
pub enum AutoGuiError {
    OSFailure(String),
    UnSupportedKey(String),
    IoError(std::io::Error),
    AliasError(String),
    OutOfBoundsError(String),
    ImageError(ImageProcessingError),
    ImgError(String),
    NulError(NulError),
    #[cfg(feature = "opencl")]
    OclError(ocl::Error),
}

impl fmt::Display for AutoGuiError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            AutoGuiError::OSFailure(err) => write!(f, "OS Failure: {}", err),
            AutoGuiError::UnSupportedKey(err) => write!(f, "Key not supported: {}", err),
            AutoGuiError::IoError(err) => write!(f, "IO Error: {}", err),
            AutoGuiError::AliasError(err) => write!(f, "Alias Error: {}", err),
            AutoGuiError::OutOfBoundsError(err) => write!(f, "Out of bounds error: {}", err),
            AutoGuiError::ImageError(err) => write!(f, "Image Error: {}", err),
            AutoGuiError::ImgError(err) => write!(f, "Image Error: {}", err),
            AutoGuiError::NulError(err) => write!(f, "Convert to C String nulerror: {}", err),
            #[cfg(feature = "opencl")]
            AutoGuiError::OclError(err) => write!(f, "OpenCL Error: {}", err),
        }
    }
}

impl From<NulError> for AutoGuiError {
    fn from(err: NulError) -> Self {
        AutoGuiError::NulError(err)
    }
}

impl From<image::ImageError> for AutoGuiError {
    fn from(err: image::ImageError) -> Self {
        AutoGuiError::ImageError(ImageProcessingError::External(err))
    }
}

impl From<ImageProcessingError> for AutoGuiError {
    fn from(err: ImageProcessingError) -> Self {
        AutoGuiError::ImageError(err)
    }
}

impl From<std::io::Error> for AutoGuiError {
    fn from(err: std::io::Error) -> Self {
        AutoGuiError::IoError(err)
    }
}
#[cfg(feature = "opencl")]
impl From<ocl::Error> for AutoGuiError {
    fn from(err: ocl::Error) -> Self {
        AutoGuiError::OclError(err)
    }
}

#[derive(Debug)]
pub enum ImageProcessingError {
    External(image::ImageError),
    Custom(String),
}

impl ImageProcessingError {
    pub fn new(msg: &str) -> Self {
        Self::Custom(msg.to_string())
    }
}

impl fmt::Display for ImageProcessingError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ImageProcessingError::External(err) => write!(f, "{}", err),
            ImageProcessingError::Custom(err) => write!(f, "{}", err),
        }
    }
}

impl std::error::Error for AutoGuiError {}
impl std::error::Error for ImageProcessingError {}
