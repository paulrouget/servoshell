use servo::compositing::windowing::WindowEvent;
use webrender_traits;
use glutin;
use std::cell::{Cell, RefCell};
use euclid::{Point2D, TypedPoint2D};

const LINE_HEIGHT: f32 = 38.0;

pub struct GlutinEventHandler {
    events_for_servo: RefCell<Vec<WindowEvent>>,
    mouse_pos: Cell<Point2D<i32>>,
}

impl GlutinEventHandler {
    pub fn new() -> GlutinEventHandler {
        GlutinEventHandler {
            events_for_servo: RefCell::new(vec!()),
            mouse_pos: Cell::new(Point2D::new(0, 0)),
        }
    }

    pub fn handle_glutin_event(&self, event: glutin::Event) -> bool {
        match event {
            glutin::Event::MouseMoved(x, y) => {
                self.mouse_pos.set(Point2D::new(x, y));
                let event = WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(x as f32, y as f32));
                self.events_for_servo.borrow_mut().push(event);
            }
            glutin::Event::Closed => {
                return true
            }
            _ => {}
        }
        false
    }

    pub fn get_events_for_servo(&self) -> Vec<WindowEvent> {
        // FIXME: why is all of that necessary?
        use std::mem;
        let mut events = mem::replace(&mut *self.events_for_servo.borrow_mut(), Vec::new());
        events.extend(mem::replace(&mut *self.events_for_servo.borrow_mut(), Vec::new()).into_iter());
        events
    }

}
