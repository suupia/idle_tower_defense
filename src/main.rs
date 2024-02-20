use bevy::{
    math::bounding::{Aabb2d, BoundingCircle, BoundingVolume, IntersectsVolume},
    prelude::*,
    sprite::MaterialMesh2dBundle,
};
use bevy::reflect::TypePath;
// use bevy_common_assets::json::JsonAssetPlugin;


mod stepping;

// Tower
const TOWER_DIAMETER: f32 = 50.;
const TOWER_ATTACK_RADIUS: f32 = 5.;
const TOWER_STARTING_POSITION: Vec3 = Vec3::new(0.0, 0.0, 0.0);
// Enemy
const ENEMY_DIAMETER: f32 = 40.;
const ENEMY_STARTING_POSITION: Vec3 = Vec3::new(100.0, 100.0, 0.0);
const ENEMY_SPEED: f32 = 150.0;

// Colors -------------------------------------
// Tower
const TOWER_COLOR: Color = Color::rgb(132.0 / 255.0, 211.0 / 255.0, 149.0 / 255.0);
const TOWER_ATTACK_RADIUS_COLOR: Color = Color::rgba(0.0, 0.0, 1.0, 0.5);
// Enemy
const ENEMY_COLOR: Color = Color::rgb(255.0 / 255.0, 0.0 / 255.0, 0.0 / 255.0);
// Button
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(bevy::log::LogPlugin {
            // Uncomment this to override the default log settings:
            level: bevy::log::Level::INFO,
            filter: "wgpu=warn,bevy_ecs=info".to_string(),
            ..default()
        }))
        .add_plugins(
            stepping::SteppingPlugin::default()
                .add_schedule(Update)
                .add_schedule(FixedUpdate)
                .at(Val::Percent(35.0), Val::Percent(50.0)),
        )
        .add_systems(Startup, setup)
        .add_systems(
            FixedUpdate,
            (move_enemy)
                // `chain`ing systems together runs them in order
                .chain(),
        )
        .add_systems(
            Update,
            (
                button_system,
                bevy::window::close_on_esc
            ),
        )
        .run();
}

#[derive(Component)]
struct Tower;

#[derive(Component)]
struct TowerAttackRadius;

#[derive(Component)]
struct Enemy;

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec2);

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut BackgroundColor,
            &mut BorderColor,
            &Children,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut text_query: Query<&mut Text>,
) {
    for (interaction, mut color, mut border_color, children) in &mut interaction_query {
        let mut text = text_query.get_mut(children[0]).unwrap();
        match *interaction {
            Interaction::Pressed => {
                text.sections[0].value = "Press".to_string();
                *color = PRESSED_BUTTON.into();
                border_color.0 = Color::RED;
                info!("Pressed");
            }
            Interaction::Hovered => {
                text.sections[0].value = "Hover".to_string();
                *color = HOVERED_BUTTON.into();
                border_color.0 = Color::WHITE;
                info!("Hovered");
            }
            Interaction::None => {
                text.sections[0].value = "Button".to_string();
                *color = NORMAL_BUTTON.into();
                border_color.0 = Color::BLACK;
                info!("None");
            }
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>) {
    // ui camera
    commands.spawn(Camera2dBundle::default());

    // ui button
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(30.0),
                height: Val::Percent(125.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        // horizontally center child text
                        justify_content: JustifyContent::Center,
                        // vertically center child text
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    border_color: BorderColor(Color::BLACK),
                    background_color: NORMAL_BUTTON.into(),
                    ..default()
                })
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Button",
                        TextStyle {
                            font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                            font_size: 40.0,
                            color: Color::rgb(0.9, 0.9, 0.9),
                        },
                    ));
                });
        });

    // Tower
    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                material: materials.add(TOWER_COLOR),
                transform: Transform::from_translation(TOWER_STARTING_POSITION)
                    .with_scale(Vec2::splat(TOWER_DIAMETER).extend(1.)),
                ..default()
            },
            Tower,
        ));
    commands
        .spawn((
            SpriteBundle {
                transform: Transform::from_translation(TOWER_STARTING_POSITION),
                  //  .with_scale(Vec2::splat(TOWER_ATTACK_RADIUS).extend(1.)),
                texture: asset_server.load("sprites/attack_range.png"),
                ..default()
            },
            TowerAttackRadius,
        ));

    // Enemy
    commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Circle::default()).into(),
                material: materials.add(ENEMY_COLOR),
                transform: Transform::from_translation(ENEMY_STARTING_POSITION)
                    .with_scale(Vec2::splat(ENEMY_DIAMETER).extend(1.)),
                ..default()
            },
            Enemy,
            Velocity(Vec2::new(1.0, 0.0) * ENEMY_SPEED),
        ));
}

fn move_enemy(
    mut query: Query<(&mut Transform, &Velocity), With<Enemy>>, time: Res<Time>,
    tower_query: Query<& Transform, (With<Tower> ,Without<Enemy>)>,
){
    for(mut transform, velocity) in &mut query{
        let tower_transform = tower_query.single();
        let direction = (tower_transform.translation - transform.translation).normalize();
        transform.translation.x += direction.x * ENEMY_SPEED * time.delta_seconds();
        transform.translation.y += direction.y * ENEMY_SPEED * time.delta_seconds();
        info!("transform.translation.x: {}", transform.translation.x);
        info!("velocity.0.x: {}",  velocity.0.x );
    }
}