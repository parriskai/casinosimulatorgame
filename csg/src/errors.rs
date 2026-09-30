//! `csg::error`
//! Game error reports
//! 
//! You probably dont need to include this if you are already using [`csg::prelude`]

use serde_json::error;
use thiserror::Error;
use vfs::VfsError;
use zip::result::ZipError;

/// An error originating or related to GLFW
#[derive(Error, Debug)]
pub enum GLFWError {
    /// Initalization error
    #[error("Failed to init ({0})")]
    InitError(glfw::InitError),

    /// Window error
    #[error("Window error ({0})")]
    WindowError(String),

    /// Error interacting with a handle
    #[error("Handle error ({0})")]
    HandleError(raw_window_handle::HandleError)
}

impl Into<GError> for glfw::InitError {
    fn into(self) -> GError {
        GError::GLFWError(GLFWError::InitError(self))
    }
}

impl Into<GError> for raw_window_handle::HandleError {
    fn into(self) -> GError {
        GError::GLFWError(GLFWError::HandleError(self))
    }
}

/// An error originating or related to WGPU
#[derive(Error, Debug)]
pub enum WGPUError{
    #[error("Failed to request an adapter ({0})")]
    RequestAdapterError(wgpu::RequestAdapterError),
    #[error("Failed to request a device ({0})")]
    RequestDeviceError(wgpu::RequestDeviceError)
}
impl Into<GError> for wgpu::RequestAdapterError{
    fn into(self) -> GError {
        GError::WGPUError(WGPUError::RequestAdapterError(self))
    }
}
impl Into<GError> for wgpu::RequestDeviceError{
    fn into(self) -> GError {
        GError::WGPUError(WGPUError::RequestDeviceError(self))
    }
}

impl Into<GError> for VfsError{
    fn into(self) -> GError {
        GError::VFSError(self)
    }
}

impl Into<GError> for ZipError{
    fn into(self) -> GError {
        GError::ZipError(self)
    }
}

impl Into<GError> for std::io::Error{
    fn into(self) -> GError {
        GError::IOError(self)
    }
}

/// General Error type
#[derive(Error, Debug)]
pub enum GError{
    #[error("GLFW Error ({0}))")]
    GLFWError(GLFWError),
    #[error("WGPU Error ({0})")]
    WGPUError(WGPUError),
    #[error("VFS Error ({0})")]
    VFSError(VfsError),
    #[error("Zip Error ({0})")]
    ZipError(ZipError),
    #[error("IO Error ({0}")]
    IOError(std::io::Error)
}

/// A thin wrapper arround rusts Result, returning either sucess or a game error
pub type GResult<T> = Result<T, GError>;

/// Helper trait, into doesnt really work for our use case so we have our own trait
pub trait GeneralizeError<T, E>{
    /// Generalize a sub error into a broad Game Error
    fn g_err(self) -> GResult<T>;
}

impl<T, E> GeneralizeError<T, E> for Result<T, E> where  E: Into<GError>{
    fn g_err(self) -> GResult<T> {
        self.map_err(|e| e.into())
    }
}