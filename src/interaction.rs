use crate::game::{Action, BoatState, Entity, EntityLocation, GameState, PlayerLocation};
use crate::world::{self, Bank, GridPos};

/// Determine what pressing E does in the current game state.
/// Returns None if no valid interaction is available.
pub fn resolve_interaction(state: &GameState) -> Option<Action> {
    match state.player {
        PlayerLocation::OnBoat => resolve_on_boat(state),
        PlayerLocation::OnLand(pos) => resolve_on_land(state, pos),
    }
}

/// When the player is on the boat.
fn resolve_on_boat(state: &GameState) -> Option<Action> {
    let BoatState::Docked(_bank) = state.boat else {
        // No interactions while crossing.
        return None;
    };

    // Priority 1: If player has a follower and boat cargo is empty, load it.
    if let Some(entity) = state.follower {
        if state.boat_cargo.is_none() {
            return Some(Action::LoadOntoBoat(entity));
        }
    }

    // Priority 2: If boat has cargo and player has no follower, unload it.
    if let Some(entity) = state.boat_cargo {
        if state.follower.is_none() {
            return Some(Action::UnloadFromBoat(entity));
        }
    }

    // Priority 3: Get off the boat.
    Some(Action::UnboardBoat)
}

/// When the player is on land.
fn resolve_on_land(state: &GameState, pos: GridPos) -> Option<Action> {
    let bank = world::bank_of(pos)?;

    let at_dock = world::is_dock_position(pos, bank)
        && state.boat == BoatState::Docked(bank);

    // Priority 1: If at dock with the boat, board it.
    if at_dock {
        // If carrying a follower, load it onto the boat instead of boarding
        // (if boat cargo is empty). This feels more natural: you walk to the dock
        // with a follower, press E to load, then press E again to board.
        if let Some(entity) = state.follower {
            if state.boat_cargo.is_none() {
                return Some(Action::LoadOntoBoat(entity));
            } else {
                return None;
            }
        }
        return Some(Action::BoardBoat);
    }

    // Priority 2: If carrying a follower, drop it.
    if let Some(entity) = state.follower {
        return Some(Action::Drop(entity));
    }

    // Priority 3: If near a free entity on the same bank, pick it up.
    if let Some(entity) = find_nearby_entity(state, pos, bank) {
        return Some(Action::PickUp(entity));
    }

    None
}

/// Find an entity on the same bank at or adjacent to the player.
/// Priority order: same tile first, then adjacent. Within each, Sheep > Wolf > Cabbage.
fn find_nearby_entity(state: &GameState, player_pos: GridPos, bank: Bank) -> Option<Entity> {
    let priority = [Entity::Sheep, Entity::Wolf, Entity::Cabbage];

    // Same tile first.
    for &entity in &priority {
        if state.follower == Some(entity) {
            continue;
        }
        if let EntityLocation::OnBank { bank: b, pos } = state.entity_location(entity) {
            if b == bank && pos == player_pos {
                return Some(entity);
            }
        }
    }

    // Adjacent tiles.
    for &entity in &priority {
        if state.follower == Some(entity) {
            continue;
        }
        if let EntityLocation::OnBank { bank: b, pos } = state.entity_location(entity) {
            if b == bank && world::is_adjacent(player_pos, pos) {
                return Some(entity);
            }
        }
    }

    None
}

/// Return a human-readable hint for what E will do.
pub fn describe_available_action(state: &GameState) -> Option<&'static str> {
    resolve_interaction(state).map(|action| match action {
        Action::PickUp(e) => match e {
            Entity::Wolf => "[E] Call wolf",
            Entity::Sheep => "[E] Call sheep",
            Entity::Cabbage => "[E] Pick up cabbage",
        },
        Action::Drop(e) => match e {
            Entity::Wolf => "[E] Send wolf away",
            Entity::Sheep => "[E] Send sheep away",
            Entity::Cabbage => "[E] Put down cabbage",
        },
        Action::LoadOntoBoat(e) => match e {
            Entity::Wolf => "[E] Load wolf onto boat",
            Entity::Sheep => "[E] Load sheep onto boat",
            Entity::Cabbage => "[E] Load cabbage onto boat",
        },
        Action::UnloadFromBoat(e) => match e {
            Entity::Wolf => "[E] Unload wolf",
            Entity::Sheep => "[E] Unload sheep",
            Entity::Cabbage => "[E] Unload cabbage",
        },
        Action::BoardBoat => "[E] Board boat",
        Action::UnboardBoat => "[E] Get off boat",
    })
}
