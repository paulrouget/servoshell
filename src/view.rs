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

    // FIXME: this event comes from the window at the moment.
    // Maybe this should come from the view
    // GeometryDidChange,

    MouseWheel(MouseScrollDelta, TouchPhase),
    // MouseInput(ElementState, MouseButton),
    // MouseMoved(i32, i32),
}

#[derive(Debug, Clone)]
pub enum TouchPhase {
    Started,
    Moved,
    Ended,
    // FIXME: Cancelled
}

// #[derive(Debug, Clone)]
// pub enum ElementState {
//     Pressed,
//     Released,
// }

// #[derive(Debug, Clone)]
// pub enum MouseButton {
//     Left,
//     Right,
// }

#[derive(Debug, Clone)]
pub enum MouseScrollDelta {
	LineDelta(f32, f32),
	PixelDelta(f32, f32)
}
