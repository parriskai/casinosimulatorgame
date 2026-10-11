use glfw::{
    Context,
    Glfw,
    GlfwReceiver,
    PWindow,
    WindowEvent
};
use crate::{commands::CommandBrodcaster, input_mgr::InputManager, prelude::*};

/// GLFW Window wrapper
pub struct Window{
    event: GlfwReceiver<(f64, WindowEvent)>,
    pub pwindow: PWindow,
    glfw: Glfw,
    cmd: CommandBrodcaster
} impl Window {
    pub fn create(cmd: CommandBrodcaster) -> GResult<Window>{
        let mut glfw = glfw::init(glfw::fail_on_errors).g_err()?;

        let (pwindow, event) = Self::create_glfw_window(&mut glfw)?;

        Ok(
            Window {
                pwindow,
                event,
                glfw,
                cmd
            }
        )
    }

    pub fn get_framebuffer_size(&self) -> WindowSize{
        let dim = self.pwindow.get_framebuffer_size();
        rect_size(dim.0.max(1) as u32, dim.1.max(1) as u32)
    }

    fn create_glfw_window(glfw: &mut glfw::Glfw) -> GResult<(PWindow, glfw::GlfwReceiver<(f64, WindowEvent)>)>{
        let (mut window, events) = glfw.create_window(
            512,
            512,
            "Casino Simulator Game",
            glfw::WindowMode::Windowed).ok_or(
                GError::GLFWError(
                    GLFWError::WindowError("Failed to create window!".into())
                )
            )?;
        window.maximize();
        window.set_all_polling(true);
        window.make_current();
        Ok((window, events))
    }

    pub fn handle_events(&mut self, input_mgr: &mut InputManager){
        self.glfw.poll_events();

        let events: Vec<_> = glfw::flush_messages(&self.event).collect();
        
        for event in events{
            match event.1 {
                WindowEvent::Close => {
                    tracing::info!("Window Close Requested");
                    self.cmd.quit_game().unwrap();
                },

                WindowEvent::Refresh => {
                    // Alredy runneing every frame
                }
                
                e => {
                    // If IM doesnt handle it
                    if !input_mgr.handle_event(&e){
                        tracing::warn!("Unhandled window event {e:?}");
                    }
                }
            }
        }

        if input_mgr.should_reload_fb{
            self.cmd.set_framebuffer_size(self.get_framebuffer_size()).unwrap()
        }
    }
}
