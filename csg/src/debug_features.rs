#[cfg(any(debug_assertions, feature = "debug_tools"))]
use crate::input_mgr::InputManager;

#[derive(Default)]
pub struct DebugFeatures{
    pub outline_quads: bool
} impl DebugFeatures{
    fn new() -> DebugFeatures{
        DebugFeatures{
            outline_quads: false
        }
    }

    #[cfg(any(debug_assertions, feature = "debug_tools"))]
    pub fn debug_hotkeys(&mut self, input_mgr: &InputManager){
        if input_mgr.key_down_for(&glfw::Key::F1) == Some(1){
            self.outline_quads ^= true;
            if self.outline_quads{
                tracing::info!("Debug feature Outline_QUADS: ENABLED")
            } else {
                tracing::info!("Debug feature Outline_QUADS: DISABLED")
            }
        }
    }
}