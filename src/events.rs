use servo::compositing::windowing::WindowEvent;
use servo::compositing::windowing::MouseWindowEvent;
use webrender_traits;
use glutin;
use std::cell::{Cell, RefCell};
use euclid::{Point2D, TypedPoint2D};
use servo::script_traits::TouchEventType;
use glutin::TouchPhase;


const LINE_HEIGHT: f32 = 38.0;

pub struct GlutinEventHandler {
    events_for_servo: RefCell<Vec<WindowEvent>>,
    mouse_pos: Cell<Point2D<i32>>,
    mouse_down_button: Cell<Option<glutin::MouseButton>>,
    mouse_down_point: Cell<Point2D<i32>>,
}

impl GlutinEventHandler {
    pub fn new() -> GlutinEventHandler {
        GlutinEventHandler {
            events_for_servo: RefCell::new(vec!()),
            mouse_pos: Cell::new(Point2D::new(0, 0)),
            mouse_down_button: Cell::new(None),
            mouse_down_point: Cell::new(Point2D::new(0, 0)),
        }
    }

    pub fn handle_glutin_event(&self, event: glutin::Event) -> bool {
        match event {
            glutin::Event::MouseMoved(x, y) => {
                self.mouse_pos.set(Point2D::new(x, y));
                let event = WindowEvent::MouseWindowMoveEventClass(TypedPoint2D::new(x as f32, y as f32));
                self.events_for_servo.borrow_mut().push(event);
            }
            glutin::Event::MouseWheel(delta, phase) => {
                let (dx, dy) = match delta {
                    glutin::MouseScrollDelta::LineDelta(dx, dy) => (dx, dy * LINE_HEIGHT),
                    glutin::MouseScrollDelta::PixelDelta(dx, dy) => (dx, dy),
                };
                let scroll_location = webrender_traits::ScrollLocation::Delta(TypedPoint2D::new(dx, dy));
                let phase = match phase {
                    TouchPhase::Started => TouchEventType::Down,
                    TouchPhase::Moved => TouchEventType::Move,
                    TouchPhase::Ended => TouchEventType::Up,
                    TouchPhase::Cancelled => TouchEventType::Cancel,
                };

                if let webrender_traits::ScrollLocation::Delta(mut delta) = scroll_location {
                    if delta.y.abs() >= delta.x.abs() {
                        delta.x = 0.0;
                    } else {
                        delta.y = 0.0;
                    }
                }

                let mouse_pos = self.mouse_pos.get();
                let event = WindowEvent::Scroll(scroll_location, TypedPoint2D::new(mouse_pos.x as i32, mouse_pos.y as i32), phase);
                self.events_for_servo.borrow_mut().push(event);
            },
            glutin::Event::MouseInput(element_state, mouse_button) => {
                if mouse_button == glutin::MouseButton::Left || mouse_button == glutin::MouseButton::Right {
                    let mouse_pos = self.mouse_pos.get();
                    use servo::script_traits::MouseButton;
                    let (x, y) = (mouse_pos.x, mouse_pos.y);
                    // FIXME(tkuehn): max pixel dist should be based on pixel density
                    let max_pixel_dist = 10f64;
                    let event = match element_state {
                        glutin::ElementState::Pressed => {
                            self.mouse_down_point.set(Point2D::new(x, y));
                            self.mouse_down_button.set(Some(mouse_button));
                            MouseWindowEvent::MouseDown(MouseButton::Left, TypedPoint2D::new(x as f32, y as f32))
                        }
                        glutin::ElementState::Released => {
                         let mouse_up_event = MouseWindowEvent::MouseUp(MouseButton::Left, TypedPoint2D::new(x as f32, y as f32));
                            match self.mouse_down_button.get() {
                                None => mouse_up_event,
                                Some(but) if mouse_button == but => {
                                    let pixel_dist = self.mouse_down_point.get() - Point2D::new(x, y);
                                    let pixel_dist = ((pixel_dist.x * pixel_dist.x +
                                                       pixel_dist.y * pixel_dist.y) as f64).sqrt();
                                    if pixel_dist < max_pixel_dist {
                                        self.events_for_servo.borrow_mut().push(WindowEvent::MouseWindowEventClass(mouse_up_event));
                                        MouseWindowEvent::Click(MouseButton::Left, TypedPoint2D::new(x as f32, y as f32))
                                    } else {
                                        mouse_up_event
                                    }
                                },
                                Some(_) => mouse_up_event,
                            }
                        }
                    };
                    self.events_for_servo.borrow_mut().push(WindowEvent::MouseWindowEventClass(event));
                }
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
