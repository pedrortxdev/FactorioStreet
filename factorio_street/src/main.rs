use bevy::prelude::*;
use bevy::window::PrimaryWindow;

// A unidade fundamental do seu império econômico
const TILE_SIZE: f32 = 64.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // Mantém o pixel art nítido
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))      // Fundo "Deep Space/Industrial"
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_movement, handle_input, update_cursor))
        .run();
}

#[derive(Component)]
struct GridCursor;

#[derive(Component)]
struct FactoryTile;

fn setup(mut commands: Commands) {
    // Câmera 2D básica
    commands.spawn(Camera2d::default());

    // Cursor visual para o "Snap-to-Grid"
    commands.spawn((
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.1), // Sombra branca transparente
            custom_size: Some(Vec2::splat(TILE_SIZE)),
            ..default()
        },
        GridCursor,
    ));
}

fn update_cursor(
    window_query: Query<&Window, With<PrimaryWindow>>,
    camera_query: Query<(&Camera, &GlobalTransform)>,
    mut cursor_query: Query<&mut Transform, With<GridCursor>>,
) {
    let Ok(window) = window_query.single() else { return; };
    let Ok((camera, camera_transform)) = camera_query.single() else { return; };

    if let Some(world_position) = window.cursor_position()
        .and_then(|cursor| camera.viewport_to_world_2d(camera_transform, cursor).ok())
    {
        let Ok(mut transform) = cursor_query.single_mut() else { return; };
        // A mágica do grid: arredonda a posição do mouse para múltiplos de 64
        transform.translation.x = (world_position.x / TILE_SIZE).round() * TILE_SIZE;
        transform.translation.y = (world_position.y / TILE_SIZE).round() * TILE_SIZE;
    }
}

fn handle_input(
    mut commands: Commands,
    mouse_input: Res<ButtonInput<MouseButton>>,
    cursor_query: Query<&Transform, With<GridCursor>>,
) {
    if mouse_input.just_pressed(MouseButton::Left) {
        let Ok(cursor_transform) = cursor_query.single() else { return; };
        
        // Spawn de um tile temporário (depois substituímos pelo seu sprite)
        commands.spawn((
            Sprite {
                color: Color::srgb(0.2, 0.5, 0.8), // Azul "Construção"
                custom_size: Some(Vec2::splat(TILE_SIZE - 2.0)), // Margem pequena pra ver o grid
                ..default()
            },
            *cursor_transform,
            FactoryTile,
        ));
        println!("Tile colocado em: {:?}", cursor_transform.translation);
    }
}

// Movimentação básica pra você explorar seu futuro mapa
fn camera_movement(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera_query: Query<&mut Transform, With<Camera>>,
    time: Res<Time>,
) {
    let Ok(mut transform) = camera_query.single_mut() else { return; };
    let speed = 500.0;
    let direction = Vec3::new(
        if keyboard_input.pressed(KeyCode::KeyD) { 1.0 } else if keyboard_input.pressed(KeyCode::KeyA) { -1.0 } else { 0.0 },
        if keyboard_input.pressed(KeyCode::KeyW) { 1.0 } else if keyboard_input.pressed(KeyCode::KeyS) { -1.0 } else { 0.0 },
        0.0,
    );
    transform.translation += direction.normalize_or_zero() * speed * time.delta_secs();
}