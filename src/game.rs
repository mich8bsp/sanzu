use crate::world::{self, Bank, Direction, GridPos};

/// The three transportable entities.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Entity {
    Wolf,
    Sheep,
    Cabbage,
}

#[allow(dead_code)]
impl Entity {
    pub const ALL: [Entity; 3] = [Entity::Wolf, Entity::Sheep, Entity::Cabbage];

    pub fn name(self) -> &'static str {
        match self {
            Entity::Wolf => "wolf",
            Entity::Sheep => "sheep",
            Entity::Cabbage => "cabbage",
        }
    }

    pub fn is_alive(self) -> bool {
        matches!(self, Entity::Wolf | Entity::Sheep)
    }
}

/// Where an entity currently is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityLocation {
    OnBank { bank: Bank, pos: GridPos },
    FollowingPlayer,
    OnBoat,
}

/// Where the player currently is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlayerLocation {
    OnLand(GridPos),
    OnBoat,
}

/// The boat's state.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BoatState {
    Docked(Bank),
    Crossing { from: Bank, progress: f32 },
}

/// High-level game phase.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GamePhase {
    Playing,
    Won,
    Lost(LoseReason),
}

/// Why the player lost.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoseReason {
    WolfAteSheep,
    SheepAteCabbage,
}

impl LoseReason {
    pub fn message(self) -> &'static str {
        match self {
            LoseReason::WolfAteSheep => "The wolf ate the sheep!",
            LoseReason::SheepAteCabbage => "The sheep ate the cabbage!",
        }
    }
}

/// All possible interaction actions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    PickUp(Entity),
    Drop(Entity),
    LoadOntoBoat(Entity),
    UnloadFromBoat(Entity),
    BoardBoat,
    UnboardBoat,
}

const CROSSING_DURATION: f32 = 2.0;

/// The full game state.
pub struct GameState {
    pub phase: GamePhase,
    pub player: PlayerLocation,
    pub entities: [(Entity, EntityLocation); 3],
    pub follower: Option<Entity>,
    pub boat: BoatState,
    pub boat_cargo: Option<Entity>,
    pub crossing_timer: f32,
    pub crossing_count: u32,
}

impl GameState {
    pub fn new() -> Self {
        Self {
            phase: GamePhase::Playing,
            player: PlayerLocation::OnLand(world::PLAYER_START),
            entities: [
                (
                    Entity::Wolf,
                    EntityLocation::OnBank {
                        bank: Bank::Left,
                        pos: world::WOLF_START,
                    },
                ),
                (
                    Entity::Sheep,
                    EntityLocation::OnBank {
                        bank: Bank::Left,
                        pos: world::SHEEP_START,
                    },
                ),
                (
                    Entity::Cabbage,
                    EntityLocation::OnBank {
                        bank: Bank::Left,
                        pos: world::CABBAGE_START,
                    },
                ),
            ],
            follower: None,
            boat: BoatState::Docked(Bank::Left),
            boat_cargo: None,
            crossing_timer: 0.0,
            crossing_count: 0,
        }
    }

    pub fn reset(&mut self) {
        *self = Self::new();
    }

    /// Get the location of a specific entity.
    pub fn entity_location(&self, entity: Entity) -> EntityLocation {
        self.entities
            .iter()
            .find(|(e, _)| *e == entity)
            .map(|(_, loc)| *loc)
            .unwrap()
    }

    /// Set the location of a specific entity.
    pub fn set_entity_location(&mut self, entity: Entity, loc: EntityLocation) {
        for (e, l) in &mut self.entities {
            if *e == entity {
                *l = loc;
                return;
            }
        }
    }

    /// Get all entities on a given bank (not following player, not on boat).
    pub fn entities_on_bank(&self, bank: Bank) -> Vec<Entity> {
        self.entities
            .iter()
            .filter_map(|(e, loc)| {
                // Exclude the entity currently following the player.
                if self.follower == Some(*e) {
                    return None;
                }
                match loc {
                    EntityLocation::OnBank { bank: b, .. } if *b == bank => Some(*e),
                    _ => None,
                }
            })
            .collect()
    }

    /// Try to move the player in a direction. Returns true if successful.
    pub fn try_move_player(&mut self, dir: Direction) -> bool {
        let PlayerLocation::OnLand(pos) = self.player else {
            return false;
        };

        let new_pos = pos.step(dir);
        if !world::is_walkable(new_pos) {
            return false;
        }

        self.player = PlayerLocation::OnLand(new_pos);

        // Move follower to the player's old position.
        if let Some(entity) = self.follower {
            self.set_entity_location(
                entity,
                EntityLocation::OnBank {
                    bank: world::bank_of(new_pos).unwrap(),
                    pos,
                },
            );
        }

        true
    }

    /// Execute an interaction action.
    pub fn execute_action(&mut self, action: Action) {
        match action {
            Action::PickUp(entity) => {
                self.follower = Some(entity);
                self.set_entity_location(entity, EntityLocation::FollowingPlayer);
            }
            Action::Drop(entity) => {
                self.follower = None;
                if let PlayerLocation::OnLand(pos) = self.player {
                    if let Some(bank) = world::bank_of(pos) {
                        self.set_entity_location(
                            entity,
                            EntityLocation::OnBank { bank, pos },
                        );
                    }
                }
            }
            Action::LoadOntoBoat(entity) => {
                self.follower = None;
                self.boat_cargo = Some(entity);
                self.set_entity_location(entity, EntityLocation::OnBoat);
            }
            Action::UnloadFromBoat(entity) => {
                self.boat_cargo = None;
                if let BoatState::Docked(bank) = self.boat {
                    let dock = world::dock_for(bank);
                    self.set_entity_location(
                        entity,
                        EntityLocation::OnBank { bank, pos: dock },
                    );
                }
            }
            Action::BoardBoat => {
                // If carrying a follower, the follower conceptually comes along
                // (stays FollowingPlayer, will be loaded next E press on boat).
                self.player = PlayerLocation::OnBoat;
            }
            Action::UnboardBoat => {
                if let BoatState::Docked(bank) = self.boat {
                    let dock = world::dock_for(bank);
                    self.player = PlayerLocation::OnLand(dock);
                    // If we had a follower, place them at the dock.
                    if let Some(entity) = self.follower {
                        self.set_entity_location(
                            entity,
                            EntityLocation::OnBank { bank, pos: dock },
                        );
                        self.follower = None;
                    }
                }
            }
        }
    }

    /// Start a river crossing. Returns true if crossing started.
    pub fn start_crossing(&mut self) -> bool {
        if self.player != PlayerLocation::OnBoat {
            return false;
        }
        let BoatState::Docked(bank) = self.boat else {
            return false;
        };

        self.boat = BoatState::Crossing {
            from: bank,
            progress: 0.0,
        };
        self.crossing_timer = 0.0;
        true
    }

    /// Update crossing animation. Call each frame with delta time.
    pub fn update_crossing(&mut self, dt: f32) {
        if let BoatState::Crossing {
            from,
            ref mut progress,
        } = self.boat
        {
            self.crossing_timer += dt;
            *progress = (self.crossing_timer / CROSSING_DURATION).min(1.0);

            if *progress >= 1.0 {
                let destination = from.opposite();
                self.boat = BoatState::Docked(destination);
                self.crossing_count += 1;
            }
        }
    }

    /// Check if any forbidden pair is left unattended.
    pub fn check_eating_rules(&self) -> Option<LoseReason> {
        let player_bank = match self.player {
            PlayerLocation::OnLand(pos) => world::bank_of(pos),
            PlayerLocation::OnBoat => None,
        };

        for bank in [Bank::Left, Bank::Right] {
            if player_bank == Some(bank) {
                continue;
            }

            let entities_here = self.entities_on_bank(bank);
            let has_wolf = entities_here.contains(&Entity::Wolf);
            let has_sheep = entities_here.contains(&Entity::Sheep);
            let has_cabbage = entities_here.contains(&Entity::Cabbage);

            if has_wolf && has_sheep {
                return Some(LoseReason::WolfAteSheep);
            }
            if has_sheep && has_cabbage {
                return Some(LoseReason::SheepAteCabbage);
            }
        }

        None
    }

    /// Check if all entities are on the right bank.
    pub fn check_win(&self) -> bool {
        self.entities.iter().all(|(_, loc)| {
            matches!(
                loc,
                EntityLocation::OnBank {
                    bank: Bank::Right,
                    ..
                }
            )
        })
    }
}
