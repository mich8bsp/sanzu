use macroquad::prelude::*;

use crate::world::Direction;

/// Events produced by the input system.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputEvent {
    Move(Direction),
    Interact,
    CrossRiver,
    Restart,
    None,
}

const INITIAL_MOVE_DELAY: f32 = 0.20;
const REPEAT_MOVE_DELAY: f32 = 0.12;

/// Tracks input state for movement cooldowns.
pub struct InputState {
    move_cooldown: f32,
    first_press: bool,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            move_cooldown: 0.0,
            first_press: true,
        }
    }

    /// Poll input this frame. Returns the highest-priority event.
    pub fn poll(&mut self, dt: f32) -> InputEvent {
        // Single-press actions take priority.
        if is_key_pressed(KeyCode::E) {
            return InputEvent::Interact;
        }
        if is_key_pressed(KeyCode::Space) {
            return InputEvent::CrossRiver;
        }
        if is_key_pressed(KeyCode::R) {
            return InputEvent::Restart;
        }

        // Movement with held-key repeat.
        if let Some(direction) = self.read_direction() {
            self.move_cooldown -= dt;
            if self.move_cooldown <= 0.0 {
                let delay = if self.first_press {
                    self.first_press = false;
                    INITIAL_MOVE_DELAY
                } else {
                    REPEAT_MOVE_DELAY
                };
                self.move_cooldown = delay;
                return InputEvent::Move(direction);
            }
        } else {
            self.move_cooldown = 0.0;
            self.first_press = true;
        }

        InputEvent::None
    }

    fn read_direction(&self) -> Option<Direction> {
        if is_key_down(KeyCode::W) || is_key_down(KeyCode::Up) {
            Some(Direction::Up)
        } else if is_key_down(KeyCode::S) || is_key_down(KeyCode::Down) {
            Some(Direction::Down)
        } else if is_key_down(KeyCode::A) || is_key_down(KeyCode::Left) {
            Some(Direction::Left)
        } else if is_key_down(KeyCode::D) || is_key_down(KeyCode::Right) {
            Some(Direction::Right)
        } else {
            None
        }
    }
}
