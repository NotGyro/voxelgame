use std::collections::HashSet;


#[derive(Clone, Default)]
pub struct InputState {
    keys_held: HashSet<::winit::VirtualKeyCode>,
    pub mouse_delta: (f64, f64),
    pub right_mouse_pressed: bool
}


impl InputState {
    pub fn new() -> InputState {
        InputState {
            keys_held: HashSet::new(),
            mouse_delta: (0.0, 0.0),
            right_mouse_pressed: false
        }
    }


    pub fn get_key_down(&self, key: &::winit::VirtualKeyCode) -> bool {
        self.keys_held.contains(key)
    }


    pub fn update_key(&mut self, input: ::winit::KeyboardInput) {
        match input.state {
            ::winit::ElementState::Pressed => {
                if let Some(keycode) = input.virtual_keycode {
                    self.keys_held.insert(keycode);
                }
            },
            ::winit::ElementState::Released => {
                if let Some(keycode) = input.virtual_keycode {
                    self.keys_held.remove(&keycode);
                }
            }
        }
    }


    pub fn add_mouse_delta(&mut self, delta: (f64, f64)) {
        self.mouse_delta = (self.mouse_delta.0 + delta.0, self.mouse_delta.1 + delta.1);
    }
}