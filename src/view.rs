pub use platform::View;

#[derive(Copy, Clone)]
pub struct DrawableGeometry {
    pub view_size: (u32, u32),
    pub margins: (u32, u32, u32, u32),
    pub position: (i32, i32),
    pub hidpi_factor: f32,
}

/// View events

#[derive(Debug, Clone)]
pub enum ViewEvent {
    GeometryDidChange,
    MouseWheel(MouseScrollDelta, TouchPhase),
    MouseInput(ElementState, MouseButton),
    MouseMoved(i32, i32),
}

#[derive(Debug, Clone)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    Cancelled
}

#[derive(Debug, Clone)]
pub enum ElementState {
    Pressed,
    Released,
}

#[derive(Debug, Clone)]
pub enum MouseButton {
    Left,
    Right,
}

#[derive(Debug, Clone)]
pub enum MouseScrollDelta {
	LineDelta(f32, f32),
	PixelDelta(f32, f32)
}
