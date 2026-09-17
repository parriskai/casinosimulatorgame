use thiserror::Error;

#[derive(Error, Debug)]
pub enum GLFWError {
    #[error("Failed to init ({0})")]
    InitError(glfw::InitError),
    #[error("Window error ({0})")]
    WindowError(String),
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


#[derive(Error, Debug)]
pub enum GError{
    #[error("GLFW Error ({0}))")]
    GLFWError(GLFWError),
    #[error("WGPU Error ({0})")]
    WGPUError(WGPUError)
}

pub type GResult<T> = Result<T, GError>;

pub trait GeneralizeError<T, E>{
    fn g_err(self) -> GResult<T>;
}

impl<T, E> GeneralizeError<T, E> for Result<T, E> where  E: Into<GError>{
    fn g_err(self) -> GResult<T> {
        self.map_err(|e| e.into())
    }
}