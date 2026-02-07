use macroquad::prelude::*;

mod anim;
mod game;
mod input;
mod interaction;
mod render;
mod world;

fn window_conf() -> Conf {
    Conf {
        window_title: "River Crossing".to_string(),
        window_width: 1920,
        window_height: 1080,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let atlas = render::SpriteAtlas::load().await;
    let mut state = game::GameState::new();
    let mut anim = anim::AnimState::new();
    let mut input_state = input::InputState::new();

    loop {
        let dt = get_frame_time();
        let time = get_time() as f32;

        // --- INPUT ---
        let event = input_state.poll(dt);

        // --- UPDATE ---
        match state.phase {
            game::GamePhase::Playing => {
                match event {
                    input::InputEvent::Move(dir) => {
                        state.try_move_player(dir);
                    }
                    input::InputEvent::Interact => {
                        if let Some(action) = interaction::resolve_interaction(&state) {
                            state.execute_action(action);
                            if state.check_win() {
                                state.phase = game::GamePhase::Won;
                            }
                        }
                    }
                    input::InputEvent::CrossRiver => {
                        if state.start_crossing() {
                            if let Some(reason) = state.check_eating_rules() {
                                state.phase = game::GamePhase::Lost(reason);
                            }
                        }
                    }
                    input::InputEvent::Restart => {
                        state.reset();
                        anim.reset();
                    }
                    input::InputEvent::None => {}
                }

                state.update_crossing(dt);
                anim.update(&state, dt);
            }
            game::GamePhase::Won | game::GamePhase::Lost(_) => {
                if event == input::InputEvent::Restart {
                    state.reset();
                    anim.reset();
                }
            }
        }

        // --- RENDER ---
        clear_background(Color::new(0.05, 0.06, 0.12, 1.0));
        render::setup_camera();
        render::draw_world(&state, &atlas, &anim, time);
        render::draw_hud(&state);

        set_default_camera();

        next_frame().await
    }
}
