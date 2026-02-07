use crate::game::{Entity, EntityLocation, GameState, PlayerLocation};
use crate::world;

const MOVE_SPEED: f32 = 350.0;
const FOLLOWER_SPEED: f32 = 300.0;
const WALK_FRAME_DURATION: f32 = 0.12;
const SNAP_DISTANCE: f32 = 128.0;
const ARRIVE_THRESHOLD: f32 = 0.5;

pub struct EntityAnim {
    pub pos: (f32, f32),
    pub moving: bool,
    pub facing_right: bool,
}

pub struct AnimState {
    pub player_pos: (f32, f32),
    pub player_moving: bool,
    pub player_facing_right: bool,
    pub walk_timer: f32,
    pub walk_frame: usize,
    pub entities: [(Entity, EntityAnim); 3],
}

impl AnimState {
    pub fn new() -> Self {
        Self {
            player_pos: world::grid_to_iso(world::PLAYER_START),
            player_moving: false,
            player_facing_right: true,
            walk_timer: 0.0,
            walk_frame: 0,
            entities: [
                (
                    Entity::Wolf,
                    EntityAnim {
                        pos: world::grid_to_iso(world::WOLF_START),
                        moving: false,
                        facing_right: true,
                    },
                ),
                (
                    Entity::Sheep,
                    EntityAnim {
                        pos: world::grid_to_iso(world::SHEEP_START),
                        moving: false,
                        facing_right: true,
                    },
                ),
                (
                    Entity::Cabbage,
                    EntityAnim {
                        pos: world::grid_to_iso(world::CABBAGE_START),
                        moving: false,
                        facing_right: true,
                    },
                ),
            ],
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    pub fn update(&mut self, state: &GameState, dt: f32) {
        // --- Player position ---
        match state.player {
            PlayerLocation::OnLand(pos) => {
                let target = world::grid_to_iso(pos);
                let dx = target.0 - self.player_pos.0;
                self.player_moving = lerp_toward(&mut self.player_pos, target, MOVE_SPEED, dt);
                if self.player_moving && dx.abs() > 0.1 {
                    self.player_facing_right = dx > 0.0;
                }
            }
            PlayerLocation::OnBoat => {
                self.player_moving = false;
            }
        }

        // --- Walk cycle timer ---
        let anyone_moving =
            self.player_moving || self.entities.iter().any(|(_, e)| e.moving);
        if anyone_moving {
            self.walk_timer += dt;
            if self.walk_timer >= WALK_FRAME_DURATION {
                self.walk_timer -= WALK_FRAME_DURATION;
                self.walk_frame = 1 - self.walk_frame;
            }
        } else {
            self.walk_frame = 0;
            self.walk_timer = 0.0;
        }

        // --- Entity positions ---
        for (entity, anim) in &mut self.entities {
            if state.follower == Some(*entity) {
                match state.player {
                    PlayerLocation::OnLand(_) => {
                        let target =
                            (self.player_pos.0 - 10.0, self.player_pos.1 + 4.0);
                        let dx = target.0 - anim.pos.0;
                        anim.moving =
                            lerp_toward(&mut anim.pos, target, FOLLOWER_SPEED, dt);
                        if dx.abs() > 0.1 {
                            anim.facing_right = dx > 0.0;
                        }
                    }
                    PlayerLocation::OnBoat => {
                        anim.moving = false;
                    }
                }
            } else {
                match state.entity_location(*entity) {
                    EntityLocation::OnBank { pos, .. } => {
                        let target = world::grid_to_iso(pos);
                        anim.moving =
                            lerp_toward(&mut anim.pos, target, MOVE_SPEED, dt);
                    }
                    _ => {
                        anim.moving = false;
                    }
                }
            }
        }
    }

    pub fn entity_anim(&self, entity: Entity) -> &EntityAnim {
        self.entities
            .iter()
            .find(|(e, _)| *e == entity)
            .map(|(_, a)| a)
            .unwrap()
    }
}

fn lerp_toward(
    current: &mut (f32, f32),
    target: (f32, f32),
    speed: f32,
    dt: f32,
) -> bool {
    let dx = target.0 - current.0;
    let dy = target.1 - current.1;
    let dist = (dx * dx + dy * dy).sqrt();

    if dist > SNAP_DISTANCE {
        *current = target;
        return false;
    }

    if dist < ARRIVE_THRESHOLD {
        *current = target;
        return false;
    }

    let step = speed * dt;
    if step >= dist {
        *current = target;
        false
    } else {
        current.0 += dx / dist * step;
        current.1 += dy / dist * step;
        true
    }
}
