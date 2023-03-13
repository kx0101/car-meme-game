use rand::{thread_rng, Rng};
use rusty_engine::prelude::*;

struct GameState {
    score: u8,
    high_score: u8,
    health_amount: u8,
    tzou_collected: u8,
    lost: bool,
}

fn main() {
    let mut game = Game::new();
    game.add_logic(game_logic);

    game.window_settings(WindowDescriptor {
        title: "TzouGame".to_string(),
        ..Default::default()
    });

    let player1 = game.add_sprite("player1", SpritePreset::RacingCarBlue);
    player1.translation.x = -500.0;
    player1.layer = 10.0;
    player1.collision = true;

    let score = game.add_text("score", "Score: 0");
    score.translation = Vec2::new(520.0, 320.0);

    let high_score = game.add_text("high_score", "High Score: 0");
    high_score.translation = Vec2::new(-520.0, 320.0);

    game.audio_manager
        .play_music(MusicPreset::WhimsicalPopsicle, 0.2);

    for i in 0..10 {
        let roadline = game.add_sprite(format!("roadline{}", i), SpritePreset::RacingBarrierWhite);
        roadline.scale = 0.1;
        roadline.translation.x = -600.0 + 150.0 * i as f32;
    }

    let obstacles = vec![
        SpritePreset::RacingBarrelBlue,
        SpritePreset::RacingBarrelRed,
        SpritePreset::RacingConeStraight,
    ];

    for (i, preset) in obstacles.into_iter().enumerate() {
        let obstacle = game.add_sprite(format!("obstacle{}", i), preset);
        obstacle.layer = 5.0;
        obstacle.collision = true;
        obstacle.translation.x = thread_rng().gen_range(800.0..1600.0);
        obstacle.translation.y = thread_rng().gen_range(-300.0..300.0);
    }

    for _i in 0..10 {
        let tzou = game.add_sprite("tzou", "tzou.png");
        tzou.layer = 10.0;
        tzou.scale = 0.2;
        tzou.collision = true;
        tzou.translation.x = thread_rng().gen_range(800.0..1600.0);
        tzou.translation.y = thread_rng().gen_range(-300.0..300.0);
    }

    let health_message = game.add_text("health_message", "Health: 3");
    health_message.translation = Vec2::new(-550.0, -320.0);

    game.run(GameState {
        high_score: 0,
        score: 0,
        health_amount: 3,
        tzou_collected: 0,
        lost: false,
    })
}

const PLAYER_SPEED: f32 = 400.0;
const ROAD_SPEED: f32 = 800.0;
fn game_logic(engine: &mut Engine, game_state: &mut GameState) {
    if game_state.lost {
        return;
    }

    let offset = ((engine.time_since_startup_f64 * 3.0).cos() * 5.0) as f32;

    let score = engine.texts.get_mut("score").unwrap();
    score.translation.x = engine.window_dimensions.x / 2.0 - 80.0;
    score.translation.y = engine.window_dimensions.y / 2.0 - 30.0 + offset;

    let health_message = engine.texts.get_mut("health_message").unwrap();
    health_message.translation.x = -engine.window_dimensions.x / 2.0 + 80.0;
    health_message.translation.y = -engine.window_dimensions.y / 2.0 + 30.0 + offset;

    let high_score = engine.texts.get_mut("high_score").unwrap();
    high_score.translation.x = -engine.window_dimensions.x / 2.0 + 110.0;
    high_score.translation.y = engine.window_dimensions.y / 2.0 - 30.0;

    let mut direction = 0.0;

    if engine
        .keyboard_state
        .pressed_any(&[KeyCode::Up, KeyCode::W])
    {
        direction += 1.0;
    }

    if engine
        .keyboard_state
        .pressed_any(&[KeyCode::Down, KeyCode::S])
    {
        direction -= 1.0;
    }

    let player1 = engine.sprites.get_mut("player1").unwrap();
    player1.translation.y += direction * PLAYER_SPEED * engine.delta_f32;
    player1.rotation = direction * 0.15;

    if player1.translation.y < -360.0 || player1.translation.y > 360.0 {
        game_state.health_amount = 0;
    }

    // Move road objects
    for sprite in engine.sprites.values_mut() {
        if sprite.label.starts_with("roadline") {
            sprite.translation.x -= ROAD_SPEED * engine.delta_f32;
            if sprite.translation.x < -675.0 {
                sprite.translation.x += 1500.0;
            }
        }

        if sprite.label.starts_with("obstacle") || sprite.label.starts_with("tzou") {
            sprite.translation.x -= ROAD_SPEED * engine.delta_f32;

            if sprite.translation.x < -800.0 {
                sprite.translation.x = thread_rng().gen_range(800.0..1600.0);
                sprite.translation.y = thread_rng().gen_range(-300.0..300.0);
            }
        }
    }

    // collisions
    for event in engine.collision_events.drain(..) {
        if event.state == CollisionState::Begin {
            if event.pair.one_starts_with("player1") && event.pair.one_starts_with("tzou") {
                // player1 collided with tzou
                engine.sprites.remove(&event.pair.1);
                game_state.score += 1;
                game_state.tzou_collected += 1;

                let score = engine.texts.get_mut("score").unwrap();
                score.value = format!("Score: {}", game_state.score);

                if game_state.score > game_state.high_score {
                    game_state.high_score = game_state.score;
                    let high_score = engine.texts.get_mut("high_score").unwrap();
                    high_score.value = format!("High Score: {}", game_state.high_score);
                }
            } else if event.pair.one_starts_with("player1") && !event.pair.one_starts_with("tzou") {
                // player1 collided with an obstacle
                for label in [event.pair.0, event.pair.1] {
                    if label != "player1" {
                        engine.sprites.remove(&label);
                    }
                }

                game_state.health_amount -= 1;
                let health_message = engine.texts.get_mut("health_message").unwrap();
                health_message.value = format!("Health: {}", game_state.health_amount);
            } else if event.pair.one_starts_with("tzou") {
                // tzou collided with something
                for label in [event.pair.0, event.pair.1] {
                    if label != "tzou" {
                        engine.sprites.remove(&label);
                    }
                }

                game_state.score += 1;
                let score = engine.texts.get_mut("score").unwrap();
                score.value = format!("Score: {}", game_state.score);

                if game_state.score > game_state.high_score {
                    game_state.high_score = game_state.score;
                    let high_score = engine.texts.get_mut("high_score").unwrap();
                    high_score.value = format!("High Score: {}", game_state.high_score);
                }
            }
        }
    }

    if game_state.tzou_collected > 0 {
        let tzou_count = engine
            .sprites
            .values()
            .filter(|s| s.label.starts_with("tzou"))
            .count();
        if tzou_count < game_state.tzou_collected as usize {
            let new_tzou = engine.add_sprite("tzou", "tzou.png");
            new_tzou.layer = 10.0;
            new_tzou.scale = 0.2;
            new_tzou.collision = true;
            new_tzou.translation.x = thread_rng().gen_range(800.0..1600.0);
            new_tzou.translation.y = thread_rng().gen_range(-300.0..300.0);
        }
    }

    if game_state.health_amount == 0 {
        game_state.lost = true;
        let game_over = engine.add_text("game over", "Game Over");
        game_over.font_size = 128.0;
        engine.audio_manager.stop_music();
        engine.audio_manager.play_sfx(SfxPreset::Jingle3, 0.5);
    }
}
