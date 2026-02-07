use macroquad::prelude::*;

use crate::anim::AnimState;
use crate::game::{BoatState, Entity, EntityLocation, GamePhase, GameState, PlayerLocation};
use crate::interaction;
use crate::world::{self, Bank, GridPos};

// ---------------------------------------------------------------------------
// Sprite atlas
// ---------------------------------------------------------------------------

pub struct SpriteAtlas {
    pub player: [Texture2D; 3], // idle, walk1, walk2
    pub wolf: [Texture2D; 3],
    pub sheep: [Texture2D; 3],
    pub cabbage: Texture2D,
    pub boat: Texture2D,
    pub tree: Texture2D,
    pub highlight: Texture2D,
}

async fn load_sprite(path: &str) -> Texture2D {
    let tex = load_texture(path).await.unwrap();
    tex.set_filter(FilterMode::Nearest);
    tex
}

impl SpriteAtlas {
    pub async fn load() -> Self {
        Self {
            player: [
                load_sprite("assets/sprites/player_idle.png").await,
                load_sprite("assets/sprites/player_walk1.png").await,
                load_sprite("assets/sprites/player_walk2.png").await,
            ],
            wolf: [
                load_sprite("assets/sprites/wolf_idle.png").await,
                load_sprite("assets/sprites/wolf_walk1.png").await,
                load_sprite("assets/sprites/wolf_walk2.png").await,
            ],
            sheep: [
                load_sprite("assets/sprites/sheep_idle.png").await,
                load_sprite("assets/sprites/sheep_walk1.png").await,
                load_sprite("assets/sprites/sheep_walk2.png").await,
            ],
            cabbage: load_sprite("assets/sprites/cabbage.png").await,
            boat: load_sprite("assets/sprites/boat.png").await,
            tree: load_sprite("assets/sprites/tree.png").await,
            highlight: load_sprite("assets/sprites/highlight.png").await,
        }
    }
}

// ---------------------------------------------------------------------------
// Camera
// ---------------------------------------------------------------------------

pub fn setup_camera() {
    let world_h = world::WORLD_HEIGHT;
    let aspect = screen_width() / screen_height();
    let world_w = world_h * aspect;

    let offset_x = (world_w - 880.0) / 2.0;

    let mut camera = Camera2D::from_display_rect(Rect {
        x: -offset_x,
        y: 0.0,
        w: world_w,
        h: world_h,
    });
    camera.zoom.y = -camera.zoom.y;
    set_camera(&camera);
}

// ---------------------------------------------------------------------------
// Drawing
// ---------------------------------------------------------------------------

pub fn draw_world(state: &GameState, atlas: &SpriteAtlas, anim: &AnimState, time: f32) {
    draw_tiles(time);
    draw_trees(atlas);
    draw_boat(state, atlas, anim);
    draw_entities(state, atlas, anim);
    draw_dock_markers(state, atlas);
}

fn draw_trees(atlas: &SpriteAtlas) {
    let tree_positions = [
        GridPos::new(0, 0),
        GridPos::new(0, 1),
        GridPos::new(0, 6),
        GridPos::new(0, 7),
        GridPos::new(1, 0),
        GridPos::new(1, 7),
        GridPos::new(11, 0),
        GridPos::new(11, 1),
        GridPos::new(11, 6),
        GridPos::new(11, 7),
        GridPos::new(10, 0),
        GridPos::new(10, 7),
    ];
    for pos in &tree_positions {
        let (x, y) = world::grid_to_iso(*pos);
        draw_sprite(&atlas.tree, x, y, 2.5);
    }
}

pub fn draw_hud(state: &GameState) {
    if state.phase == GamePhase::Playing {
        if let Some(hint) = interaction::describe_available_action(state) {
            draw_text_centered(hint, 440.0, world::WORLD_HEIGHT - 20.0, 22.0, WHITE);
        }

        if state.player == PlayerLocation::OnBoat {
            if let BoatState::Docked(_) = state.boat {
                draw_text_centered(
                    "[SPACE] Cross river",
                    440.0,
                    world::WORLD_HEIGHT - 42.0,
                    20.0,
                    YELLOW,
                );
            }
        }

        let count_text = format!("Crossings: {}", state.crossing_count);
        draw_text(&count_text, 750.0, 18.0, 20.0, WHITE);

        draw_text(
            "WASD: Move   E: Interact   R: Restart",
            10.0,
            18.0,
            16.0,
            GRAY,
        );
    }

    match state.phase {
        GamePhase::Won => {
            draw_rectangle(
                0.0,
                world::WORLD_HEIGHT / 2.0 - 50.0,
                900.0,
                100.0,
                Color::new(0.0, 0.2, 0.0, 0.85),
            );
            draw_text_centered(
                "All items across! You win!",
                440.0,
                world::WORLD_HEIGHT / 2.0 - 5.0,
                28.0,
                GREEN,
            );
            draw_text_centered(
                "[R] Play again",
                440.0,
                world::WORLD_HEIGHT / 2.0 + 25.0,
                20.0,
                WHITE,
            );
        }
        GamePhase::Lost(reason) => {
            draw_rectangle(
                0.0,
                world::WORLD_HEIGHT / 2.0 - 50.0,
                900.0,
                100.0,
                Color::new(0.2, 0.0, 0.0, 0.85),
            );
            draw_text_centered(
                reason.message(),
                440.0,
                world::WORLD_HEIGHT / 2.0 - 5.0,
                28.0,
                RED,
            );
            draw_text_centered(
                "[R] Try again",
                440.0,
                world::WORLD_HEIGHT / 2.0 + 25.0,
                20.0,
                WHITE,
            );
        }
        GamePhase::Playing => {}
    }
}

fn draw_text_centered(text: &str, cx: f32, cy: f32, font_size: f32, color: Color) {
    let dims = measure_text(text, None, font_size as u16, 1.0);
    draw_text(text, cx - dims.width / 2.0, cy, font_size, color);
}

// ---------------------------------------------------------------------------
// Tiles
// ---------------------------------------------------------------------------

fn draw_tiles(time: f32) {
    for depth in 0..=(world::GRID_COLS + world::GRID_ROWS - 2) {
        for col in 0..world::GRID_COLS {
            let row = depth - col;
            if row < 0 || row >= world::GRID_ROWS {
                continue;
            }
            let pos = GridPos::new(col, row);

            if col >= world::RIVER_COL_MIN && col <= world::RIVER_COL_MAX {
                draw_water_tile(pos, time);
            } else {
                draw_land_tile(pos);
            }
        }
    }
}

fn draw_land_tile(pos: GridPos) {
    let (cx, cy) = world::grid_to_iso(pos);
    let hw = world::TILE_WIDTH / 2.0;
    let hh = world::TILE_HEIGHT / 2.0;

    let (base, dark) = if (pos.col + pos.row) % 2 == 0 {
        (
            Color::new(0.35, 0.70, 0.25, 1.0),
            Color::new(0.28, 0.58, 0.18, 1.0),
        )
    } else {
        (
            Color::new(0.30, 0.63, 0.22, 1.0),
            Color::new(0.25, 0.52, 0.16, 1.0),
        )
    };

    let is_edge = pos.col == world::LEFT_BANK_COL_MAX || pos.col == world::RIGHT_BANK_COL_MIN;
    let color = if is_edge {
        Color::new(0.55, 0.45, 0.28, 1.0)
    } else {
        base
    };
    let outline = if is_edge {
        Color::new(0.45, 0.38, 0.22, 1.0)
    } else {
        dark
    };

    let top = vec2(cx, cy - hh);
    let right = vec2(cx + hw, cy);
    let bottom = vec2(cx, cy + hh);
    let left = vec2(cx - hw, cy);

    draw_triangle(top, right, bottom, color);
    draw_triangle(top, left, bottom, color);

    draw_line(top.x, top.y, right.x, right.y, 1.0, outline);
    draw_line(right.x, right.y, bottom.x, bottom.y, 1.0, outline);
    draw_line(bottom.x, bottom.y, left.x, left.y, 1.0, outline);
    draw_line(left.x, left.y, top.x, top.y, 1.0, outline);
}

fn draw_water_tile(pos: GridPos, time: f32) {
    let (cx, cy) = world::grid_to_iso(pos);
    let hw = world::TILE_WIDTH / 2.0;
    let hh = world::TILE_HEIGHT / 2.0;

    let wave = ((time * 1.5 + pos.col as f32 * 0.7 + pos.row as f32 * 0.5).sin() * 0.06).abs();
    let color = Color::new(0.12 + wave, 0.30 + wave * 0.5, 0.65, 1.0);
    let outline = Color::new(0.08, 0.22, 0.50, 1.0);

    let top = vec2(cx, cy - hh);
    let right = vec2(cx + hw, cy);
    let bottom = vec2(cx, cy + hh);
    let left = vec2(cx - hw, cy);

    draw_triangle(top, right, bottom, color);
    draw_triangle(top, left, bottom, color);

    let wave_offset = (time * 2.0 + pos.col as f32 + pos.row as f32).sin() * 2.0;
    let wave_color = Color::new(0.25, 0.50, 0.80, 0.4);
    draw_line(
        cx - hw * 0.4,
        cy + wave_offset,
        cx + hw * 0.4,
        cy + wave_offset - 1.0,
        0.8,
        wave_color,
    );

    draw_line(top.x, top.y, right.x, right.y, 0.5, outline);
    draw_line(right.x, right.y, bottom.x, bottom.y, 0.5, outline);
    draw_line(bottom.x, bottom.y, left.x, left.y, 0.5, outline);
    draw_line(left.x, left.y, top.x, top.y, 0.5, outline);
}

// ---------------------------------------------------------------------------
// Dock markers
// ---------------------------------------------------------------------------

fn draw_dock_markers(state: &GameState, atlas: &SpriteAtlas) {
    if let BoatState::Docked(bank) = state.boat {
        let dock = world::dock_for(bank);
        let (x, y) = world::grid_to_iso(dock);
        draw_sprite(&atlas.highlight, x, y, 2.0);
    }
}

// ---------------------------------------------------------------------------
// Boat
// ---------------------------------------------------------------------------

fn draw_boat(state: &GameState, atlas: &SpriteAtlas, anim: &AnimState) {
    let (bx, by) = boat_screen_pos(state);
    draw_sprite(&atlas.boat, bx, by, 2.5);

    // Draw cargo on the boat (idle frame)
    if let Some(entity) = state.boat_cargo {
        let tex = entity_frame(atlas, entity, 0);
        draw_sprite(tex, bx, by - 8.0, 1.8);
    }

    // Draw player on the boat (idle frame)
    if state.player == PlayerLocation::OnBoat {
        draw_sprite(&atlas.player[0], bx + 6.0, by - 10.0, 2.0);

        // Draw follower on the boat
        if let Some(entity) = state.follower {
            let tex = entity_frame(atlas, entity, 0);
            draw_sprite(tex, bx - 6.0, by - 8.0, 1.8);
        }
    }
}

fn boat_screen_pos(state: &GameState) -> (f32, f32) {
    match state.boat {
        BoatState::Docked(bank) => boat_dock_pos(bank),
        BoatState::Crossing { from, progress } => {
            let (fx, fy) = boat_dock_pos(from);
            let (tx, ty) = boat_dock_pos(from.opposite());
            let t = smooth_step(progress);
            (fx + (tx - fx) * t, fy + (ty - fy) * t)
        }
    }
}

fn boat_dock_pos(bank: Bank) -> (f32, f32) {
    let dock = world::dock_for(bank);
    let river_col = match bank {
        Bank::Left => world::RIVER_COL_MIN,
        Bank::Right => world::RIVER_COL_MAX,
    };
    let river_pos = GridPos::new(river_col, world::DOCK_ROW);
    let (dx, dy) = world::grid_to_iso(dock);
    let (rx, ry) = world::grid_to_iso(river_pos);
    ((dx + rx) / 2.0, (dy + ry) / 2.0)
}

fn smooth_step(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

// ---------------------------------------------------------------------------
// Entities & Player (animated, depth-sorted)
// ---------------------------------------------------------------------------

#[derive(Clone, Copy)]
enum Drawable {
    Entity(Entity),
    Player,
}

struct DrawCmd {
    depth: f32,
    drawable: Drawable,
    x: f32,
    y: f32,
    scale: f32,
    flip_x: bool,
    frame: usize,
}

fn draw_entities(state: &GameState, atlas: &SpriteAtlas, anim: &AnimState) {
    let mut cmds: Vec<DrawCmd> = Vec::new();

    for &(entity, _loc) in &state.entities {
        // Skip entities rendered by the boat
        if state.boat_cargo == Some(entity) {
            continue;
        }
        if state.follower == Some(entity) && state.player == PlayerLocation::OnBoat {
            continue;
        }

        let ea = anim.entity_anim(entity);
        let frame = if ea.moving { 1 + anim.walk_frame } else { 0 };
        let flip = !ea.facing_right;

        cmds.push(DrawCmd {
            depth: ea.pos.1,
            drawable: Drawable::Entity(entity),
            x: ea.pos.0,
            y: ea.pos.1,
            scale: 2.0,
            flip_x: flip,
            frame,
        });
    }

    // Player on land
    if let PlayerLocation::OnLand(_) = state.player {
        let frame = if anim.player_moving {
            1 + anim.walk_frame
        } else {
            0
        };
        cmds.push(DrawCmd {
            depth: anim.player_pos.1,
            drawable: Drawable::Player,
            x: anim.player_pos.0,
            y: anim.player_pos.1,
            scale: 2.0,
            flip_x: !anim.player_facing_right,
            frame,
        });
    }

    cmds.sort_by(|a, b| a.depth.partial_cmp(&b.depth).unwrap());

    for cmd in &cmds {
        let tex = match cmd.drawable {
            Drawable::Entity(e) => entity_frame(atlas, e, cmd.frame),
            Drawable::Player => &atlas.player[cmd.frame],
        };
        let bob = if cmd.frame > 0 { -1.5 } else { 0.0 };
        draw_sprite_ex(tex, cmd.x, cmd.y + bob, cmd.scale, cmd.flip_x);
    }
}

fn entity_frame<'a>(atlas: &'a SpriteAtlas, entity: Entity, frame: usize) -> &'a Texture2D {
    match entity {
        Entity::Wolf => &atlas.wolf[frame],
        Entity::Sheep => &atlas.sheep[frame],
        Entity::Cabbage => &atlas.cabbage,
    }
}

// ---------------------------------------------------------------------------
// Sprite drawing helpers
// ---------------------------------------------------------------------------

fn draw_sprite(texture: &Texture2D, iso_x: f32, iso_y: f32, scale: f32) {
    draw_sprite_ex(texture, iso_x, iso_y, scale, false);
}

fn draw_sprite_ex(texture: &Texture2D, iso_x: f32, iso_y: f32, scale: f32, flip_x: bool) {
    let dest_w = texture.width() * scale;
    let dest_h = texture.height() * scale;
    let draw_x = iso_x - dest_w / 2.0;
    let draw_y = iso_y - dest_h;

    draw_texture_ex(
        texture,
        draw_x,
        draw_y,
        WHITE,
        DrawTextureParams {
            dest_size: Some(vec2(dest_w, dest_h)),
            flip_x,
            ..Default::default()
        },
    );
}
