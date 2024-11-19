use bevy::prelude::*;
use bevy_embedded_assets::EmbeddedAssetPlugin;
use rand::prelude::*;
use std::env;

#[derive(Component)]
struct Velocity(Vec2);

#[derive(Resource)]
struct WindowSize {
    width: f32,
    height: f32,
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let sprite_count = args
        .iter()
        .position(|arg| arg == "--count")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(120);

    let sprite_size = args
        .iter()
        .position(|arg| arg == "--size")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(12.0);

    let sprite_speed = args
        .iter()
        .position(|arg| arg == "--speed")
        .and_then(|index| args.get(index + 1))
        .and_then(|value| value.parse::<f32>().ok())
        .unwrap_or(240.0);

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "gleepers".to_string(),
                resolution: (444.4, 444.4).into(),
                resizable: true,
                ..default()
            }),
            ..default()
        }))
        .add_plugins(EmbeddedAssetPlugin::default())
        .insert_resource(WindowSize {
            width: 444.4,
            height: 444.4,
        })
        .insert_resource(SpriteSettings {
            count: sprite_count,
            size: sprite_size,
            speed: sprite_speed,
        })
        .add_systems(Startup, setup)
        .add_systems(Update, update_window_size)
        .add_systems(Update, sprite_movement)
        .add_systems(Update, handle_collisions)
        .run();
}

#[derive(Resource)]
struct SpriteSettings {
    count: usize,
    size: f32,
    speed: f32,
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    window_size: Res<WindowSize>,
    sprite_settings: Res<SpriteSettings>,
) {
    commands.spawn(Camera2dBundle::default());

    let texture_handle = asset_server.load("embedded://silly.png");

    let mut rng = rand::thread_rng();
    for _ in 0..sprite_settings.count {
        let x = rng.gen_range(-window_size.width / 2.0..window_size.width / 2.0);
        let y = rng.gen_range(-window_size.height / 2.0..window_size.height / 2.0);
        let velocity = Vec2::new(rng.gen_range(-1.0..1.0), rng.gen_range(-1.0..1.0)).normalize()
            * sprite_settings.speed;

        commands.spawn((
            SpriteBundle {
                texture: texture_handle.clone(),
                transform: Transform {
                    translation: Vec3::new(x, y, 0.0),
                    scale: Vec3::splat(sprite_settings.size / 128.0),
                    ..default()
                },
                ..default()
            },
            Velocity(velocity),
        ));
    }
}

fn update_window_size(mut window_size: ResMut<WindowSize>, windows: Query<&Window>) {
    if let Ok(window) = windows.get_single() {
        window_size.width = window.width();
        window_size.height = window.height();
    }
}

fn sprite_movement(
    time: Res<Time>,
    window_size: Res<WindowSize>,
    sprite_settings: Res<SpriteSettings>,
    mut query: Query<(&mut Transform, &mut Velocity)>,
) {
    let half_width = window_size.width / 2.0;
    let half_height = window_size.height / 2.0;

    for (mut transform, mut velocity) in query.iter_mut() {
        transform.translation.x += velocity.0.x * time.delta_seconds();
        transform.translation.y += velocity.0.y * time.delta_seconds();

        if transform.translation.x - sprite_settings.size / 2.0 < -half_width
            || transform.translation.x + sprite_settings.size / 2.0 > half_width
        {
            velocity.0.x = -velocity.0.x;
            transform.translation.x = transform.translation.x.clamp(
                -half_width + sprite_settings.size / 2.0,
                half_width - sprite_settings.size / 2.0,
            );
        }

        if transform.translation.y - sprite_settings.size / 2.0 < -half_height
            || transform.translation.y + sprite_settings.size / 2.0 > half_height
        {
            velocity.0.y = -velocity.0.y;
            transform.translation.y = transform.translation.y.clamp(
                -half_height + sprite_settings.size / 2.0,
                half_height - sprite_settings.size / 2.0,
            );
        }
    }
}

fn handle_collisions(
    mut query: Query<(&mut Transform, &mut Velocity)>,
    sprite_settings: Res<SpriteSettings>,
) {
    let mut entities = query.iter_combinations_mut();

    while let Some([(mut transform_a, mut velocity_a), (mut transform_b, mut velocity_b)]) =
        entities.fetch_next()
    {
        let delta = transform_a.translation - transform_b.translation;
        if delta.length() < sprite_settings.size {
            let temp = velocity_a.0;
            velocity_a.0 = velocity_b.0;
            velocity_b.0 = temp;

            let offset = delta.normalize() * sprite_settings.size * 0.5;
            transform_a.translation += offset;
            transform_b.translation -= offset;
        }
    }
}
