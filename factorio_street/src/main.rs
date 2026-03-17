use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use std::collections::HashMap; // Usando std por enquanto para garantir compilação

#[derive(Resource, Default)]
struct TileMap {
    // Mapeia uma coordenada (x, y) para a Entidade que está lá
    map: HashMap<IVec2, Entity>,
}

#[derive(Resource)]
struct GameAssets {
    belt_base: Handle<Image>,
    belt_moving: Handle<Image>,
}

#[derive(Component)]
struct ScrollingPart;

// A unidade fundamental do seu império econômico
const TILE_SIZE: f32 = 64.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(ImagePlugin::default_nearest())) // Mantém o pixel art nítido
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))      // Fundo "Deep Space/Industrial"
        .init_resource::<TileMap>()
        .add_systems(Startup, setup)
        .add_systems(Update, (camera_movement, handle_input, update_cursor))
        .run();
}

#[derive(Component)]
struct GridCursor;

#[derive(Component)]
struct FactoryTile;

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Câmera 2D básica
    commands.spawn(Camera2d::default());

    // Carrega as imagens uma única vez
    commands.insert_resource(GameAssets {
        belt_base: asset_server.load("conveyor_base.png"),
        belt_moving: asset_server.load("conveyor_moving.png"),
    });

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
    mut tile_map: ResMut<TileMap>, // Acessamos nosso "Registro de Imóveis"
    assets: Res<GameAssets>,       // Usamos nossos assets pré-carregados
) {
    let Ok(cursor_transform) = cursor_query.single() else { return; };
    
    // Converte a posição visual para coordenadas inteiras do grid
    let grid_x = (cursor_transform.translation.x / TILE_SIZE).round() as i32;
    let grid_y = (cursor_transform.translation.y / TILE_SIZE).round() as i32;
    let grid_pos = IVec2::new(grid_x, grid_y);

    // CONSTRUIR (Botão Esquerdo)
    if mouse_input.just_pressed(MouseButton::Left) {
        if !tile_map.map.contains_key(&grid_pos) {
            // Criamos uma entidade "pai" para agrupar as camadas
            // No Bevy 0.15+, apenas spawnamos os componentes necessários (Transform + Visibility)
            let entity = commands.spawn((
                *cursor_transform,
                Visibility::default(),
                FactoryTile,
            )).with_children(|parent| {
                // Camada 1: Base Estática
                parent.spawn(Sprite {
                    image: assets.belt_base.clone(),
                    custom_size: Some(Vec2::splat(TILE_SIZE)),
                    ..default()
                });

                // Camada 2: Setas Animadas (Z um pouco maior)
                parent.spawn((
                    Sprite {
                        image: assets.belt_moving.clone(),
                        custom_size: Some(Vec2::splat(TILE_SIZE)),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, 0.1), // Offset em relação ao pai
                    ScrollingPart, // Tag para animar depois
                ));
            }).id();

            tile_map.map.insert(grid_pos, entity); // Salva no mapa
            println!("Construído em: {:?}", grid_pos);
        } else {
            println!("Ocupado! O capitalismo não permite invasão de terra.");
        }
    }

    // DELETAR (Botão Direito)
    if mouse_input.just_pressed(MouseButton::Right) {
        if let Some(entity) = tile_map.map.remove(&grid_pos) {
            commands.entity(entity).despawn(); // Usando despawn() para compatibilidade
            println!("Tile removido em: {:?}", grid_pos);
        }
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