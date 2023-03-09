use std::collections::VecDeque;

use bevy::{prelude::*, time::Stopwatch};
use rand::prelude::*;

const FRAMES: usize = 6572;
const FPS: f64 = 30.0;
const WIDTH: u32 = 480;
const HEIGHT: u32 = 360;

#[derive(Resource)]
pub enum State {
    Paused,
    Playing,
}

#[derive(Component)]
struct Player {
    buffer: VecDeque<Handle<Image>>,
    play_index: usize,
    load_index: usize,
    time: Stopwatch,
}

fn update_sprite(mut player: Query<(&mut Player, &mut Handle<Image>)>, time: Res<Time>) {
    let (mut player, mut image) = player.single_mut();

    player.time.tick(time.delta());

    let current_idx = (player.time.elapsed_secs_f64() / (1.0 / FPS)).floor() as usize;
    if player.play_index < current_idx {
        if let Some(new_frame) = player.buffer.pop_front() {
            *image = new_frame;
            player.play_index += 1;
        }
    }
}

fn load_frames(mut player: Query<&mut Player>, server: Res<AssetServer>) {
    let mut player = player.single_mut();

    if player.load_index >= FRAMES {
        return;
    }

    while player.buffer.len() < 256 {
        let idx = player.load_index;
        player
            .buffer
            .push_back(server.load(format!("frames/out{idx:04}.png")));
        player.load_index += 1;
    }
}

#[derive(Component)]
struct Particle(usize);

fn color_particle(mut particles: Query<(&Particle, &mut Sprite)>, player: Query<&Player>) {
    let player = player.single();

    for (standstill, mut sprite) in &mut particles {
        let diff = player.play_index - standstill.0;

        sprite.color = if diff == 0 {
            Color::BLACK
        } else {
            // negative exponential for color transition
            Color::rgb(1.0 - (-(diff as f32) / 12.0).exp(), 0.0, 0.0)
        }
    }
}

fn move_particle(
    mut particles: Query<(&mut Transform, &mut Particle)>,
    images: Res<Assets<Image>>,
    player: Query<(&Handle<Image>, &Player)>,
) {
    let (player_image, player) = player.single();

    if let Some(image) = images.get(player_image) {
        if image.texture_descriptor.size.width != WIDTH {
            return;
        }
        if image.texture_descriptor.size.height != HEIGHT {
            return;
        }

        let block_size = image.texture_descriptor.format.describe().block_size;

        particles
            .iter_mut()
            .for_each(|(mut particle, mut standstill)| {
                let mut rng = thread_rng();

                let pos = particle.translation.truncate() + Vec2::new(240.0, 180.0);
                let mut pos = pos.as_uvec2();
                pos.y = (HEIGHT - 1).saturating_sub(pos.y);

                let idx = pos.y.clamp(0, HEIGHT - 1) * WIDTH + pos.x.clamp(0, WIDTH - 1);
                let color = image.data[idx as usize * block_size as usize];

                if color > 128 {
                    // if on opposite color, move randomly
                    particle.translation +=
                        Vec2::new(rng.gen_range(-5..=5) as f32, rng.gen_range(-5..=5) as f32)
                            .extend(0.0);
                } else {
                    standstill.0 = player.play_index;
                }

                if particle.translation.x < -240.0 {
                    particle.translation.x = 240.0
                }
                if particle.translation.x >= 240.0 {
                    particle.translation.x = -240.0
                }
                if particle.translation.y < -180.0 {
                    particle.translation.y = 180.0
                }
                if particle.translation.y >= 180.0 {
                    particle.translation.y = -180.0
                }
            });
    }
}

pub fn is_playing(state: Res<State>) -> bool {
    matches!(*state, State::Playing)
}

pub fn set_state(mut state: ResMut<State>, keyboard: Res<Input<KeyCode>>) {
    if keyboard.just_released(KeyCode::Space) {
        *state = match *state {
            State::Playing => State::Paused,
            State::Paused => State::Playing,
        }
    }
}

#[derive(Resource)]
struct MusicPlayer(Handle<AudioSink>);

fn play_audio(music_player: Res<MusicPlayer>, sinks: Res<Assets<AudioSink>>, state: Res<State>) {
    if let Some(sink) = sinks.get(&music_player.0) {
        match *state {
            State::Playing => sink.play(),
            State::Paused => sink.pause(),
        }
    }
}

fn startup(
    mut commands: Commands,
    server: Res<AssetServer>,
    audio: Res<Audio>,
    mut music_player: ResMut<MusicPlayer>,
    sinks: Res<Assets<AudioSink>>,
) {
    commands.spawn(Camera2dBundle::default());

    let player = Player {
        buffer: VecDeque::new(),
        play_index: 0,
        load_index: 1,
        time: Stopwatch::new(),
    };

    commands.spawn((
        player,
        SpriteBundle {
            sprite: Sprite {
                // color: Color::GRAY,
                ..default()
            },
            visibility: Visibility::Hidden,
            ..default()
        },
    ));

    let texture = server.load("particle.png");
    for _ in 0..30000 {
        commands.spawn((
            Particle(0),
            SpriteBundle {
                sprite: Sprite {
                    color: Color::BLACK,
                    custom_size: Some(Vec2::new(2.0, 2.0)),
                    ..default()
                },
                texture: texture.clone(),
                transform: Transform::from_xyz(
                    thread_rng().gen_range(-240..240) as f32,
                    thread_rng().gen_range(-180..180) as f32,
                    5.0,
                ),
                ..default()
            },
        ));
    }

    let handle = audio.play(server.load("bad_apple.ogg"));
    music_player.0 = sinks.get_handle(handle);
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::WHITE))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                resolution: (WIDTH as f32, HEIGHT as f32).into(),
                ..default()
            }),
            ..default()
        }))
        .add_startup_system(startup)
        .insert_resource(State::Paused)
        .add_system(set_state)
        .insert_resource(MusicPlayer(Handle::default()))
        .add_system(play_audio)
        .add_system(load_frames)
        .add_system(update_sprite.run_if(is_playing))
        .add_system(move_particle.run_if(is_playing))
        // .add_system(color_particle.run_if(is_playing))
        .run();
}
