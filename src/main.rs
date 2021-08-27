use bevy::{prelude::*};
use rand::Rng;

const WIN_H: f32 = 640.;
const WIN_W: f32 = 1024.;
const SHIP_H: f32 = 0.1;

fn main() {
    App::build()
    .insert_resource(ClearColor(Color::WHITE))
    .insert_resource(WindowDescriptor {
        title: "Invaders".to_string(),
        width: WIN_W,
        height: WIN_H,
        // vsync: true,
        // resizable: false,
        ..Default::default()
    })
    .add_plugins(DefaultPlugins)
    .add_startup_system(setup.system())
    .add_plugin(PlayerPlugin)

    .add_state(GameState::SetupInvaders)
    .add_system_set(
        SystemSet::on_enter(GameState::SetupInvaders).with_system(spawn_invaders.system())
    )

    .add_system_set(
        SystemSet::on_update(GameState::Battle)
        .with_system(weapons.system())
        .with_system(bullet_collisions.system())
        .with_system(animate_sprites.system())
        .with_system(physics.system())
        .with_system(location_despawn.system())
        .with_system(player_input.system())
        .with_system(ai.system())
        .with_system(detect_end_of_wave.system())
    )

    .run();
}

struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_stage(
            "player_spawn",
            SystemStage::single(spawn_ship.system()),
        );
    }
}

#[derive(Clone, Eq, PartialEq, Debug, Hash)]
enum GameState {
    SetupInvaders,
    Battle,
    Dead
}

pub struct Materials {
    ship: Handle<TextureAtlas>,
    bullet: Handle<ColorMaterial>,
}

struct Wave {
    level: u32,
    enemies: u32,
}

#[derive(Debug)]
enum AIStatus {
    Move,   
}

#[derive(Debug)]
struct Ship {}

#[derive(Debug)]
struct Health {
    level: f32
}

#[derive(Debug,Default)]
struct Physics {
    thrust:  Vec3,
    max_speed: Vec3,
    acceleration: Vec3,
    velocity: Vec3,
    drag: Vec3
}   

impl Physics {

    fn update(&mut self, time: &Res<Time>) {
        // get acceleration from thrust
        let mut acceleration = self.thrust * self.acceleration;

        // apply drag based on velocity
        let abs_drag = Vec3::select(
            self.velocity.cmpne(Vec3::ZERO),
            self.drag, Vec3::ZERO
        ).floor();
        let drag = Vec3::select(
            self.velocity.cmplt(Vec3::ZERO),
            abs_drag, -abs_drag
        );
        acceleration += drag;

        // apply net acceleration
        let delta_velocity: Vec3 = acceleration * time.delta_seconds();
        let velocity: Vec3 = self.velocity + delta_velocity;
        if velocity.abs() < self.max_speed {
            self.velocity = velocity;
        }
    }
}

#[derive(Debug)]
struct Weapon {
    fired: bool,
    offset: Vec3,
    cooldown: Timer,
    facing: f32,
}

#[derive(Debug)]
struct Bullet {
    hit_damage: f32
}

#[derive(Debug)]
struct Player {}

#[derive(Debug)]
struct Enemy {}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());

    let ship_texture_atlas = TextureAtlas::from_grid(
        asset_server.load("../assets/ship.png"), 
        Vec2::new(32.0, 32.0), 5, 1
    );

    let bullet_material = materials.add(asset_server.load("../assets/bullet.png").into());

    commands.insert_resource(
        Materials {
            ship : texture_atlases.add(ship_texture_atlas),
            bullet : bullet_material,
        }
    );

    commands.insert_resource(
        Wave {level: 1, enemies: 0}
    )

}

fn detect_end_of_wave(
    mut state: ResMut<State<GameState>>,
    mut wave: ResMut<Wave>,
) {
    if wave.enemies < 1 {
       // complete invaders setup phase
       state.set(GameState::SetupInvaders).unwrap();
       wave.level += 1;
    }
}

fn spawn_ship(
    mut commands: Commands,
    materials: Res<Materials>,
) {
    commands
    .spawn_bundle(SpriteSheetBundle {
        texture_atlas: materials.ship.clone(),
        transform: Transform {
            translation: Vec3::new(0.0, -215.0, 0.0),
            scale: Vec3::new(2.0, 2.0, 0.0),
            ..Default::default()
        },
        ..Default::default()
    })
    .insert(Physics {
        thrust: Vec3::ZERO,
        max_speed: Vec3::new(1000.0, 400.0, 0.0),
        acceleration: Vec3::new(4000.0, 3000.0, 0.0),
        velocity: Vec3::ZERO,
        drag: Vec3::new(2800.0, 2800.0, 0.0)
    })
    .insert(Timer::from_seconds(0.1, true))
    .insert(Player {})
    .insert(Ship {})
    .insert(Health { level: 100.0 })
    .insert(Weapon {
        fired: false,
        offset: Vec3::new(0.0, 30.0, 0.0),
        cooldown: Timer::from_seconds(0.2, false),
        facing: 1.0,
    });
}

fn spawn_invaders(
    mut commands: Commands,
    mut state: ResMut<State<GameState>>,
    mut wave: ResMut<Wave>,
    materials: Res<Materials>,
) {
    for _ in 1..wave.level {
        commands
        .spawn_bundle(SpriteSheetBundle {
            texture_atlas: materials.ship.clone(),
            transform: Transform {
                translation: Vec3::new(0.0, 215.0, 0.0),
                scale: Vec3::new(2.0, 2.0, 0.0),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Timer::from_seconds(0.1, true))
        .insert(Enemy {})
        .insert(Ship {})
        .insert(Health { level: 20.0 })
        .insert(Physics {
            thrust: Vec3::ZERO,
            max_speed: Vec3::new(1000.0, 400.0, 0.0),
            acceleration: Vec3::new(4000.0, 3000.0, 0.0),
            velocity: Vec3::ZERO,
            drag: Vec3::new(2800.0, 2800.0, 0.0)
        })
        .insert(Weapon {
            fired: false,
            offset: Vec3::new(0.0, -20.0, 0.0),
            cooldown: Timer::from_seconds(1.0, false),
            facing: -1.0,
        });

        wave.enemies += 1;
    }

    
    // complete invaders setup phase
    state.set(GameState::Battle).unwrap();
}

fn player_input(
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Physics, &mut Weapon), With<Player>>
) {
    if let Ok((mut physics, mut gun)) = query.single_mut() {
        let mut thrust = Vec3::ZERO;
        if keyboard_input.pressed(KeyCode::A) | keyboard_input.pressed(KeyCode::Left) {
            thrust -= Vec3::X;
        }

        if keyboard_input.pressed(KeyCode::D) | keyboard_input.pressed(KeyCode::Right) {
            thrust += Vec3::X;
        }

        physics.thrust = thrust;

        if keyboard_input.pressed(KeyCode::Space) {
            gun.fired = true;
        }
    }
}

fn physics(
    time: Res<Time>,
    mut query: Query<(&mut Physics, &mut Transform)>
) {
    for (mut physics, mut location) in query.iter_mut() {
        physics.update(&time);
        location.translation += physics.velocity * time.delta_seconds();
        // bound the ship within the walls
        location.translation.x = location.translation.x.min(500.0).max(-500.0);
    }
}

fn is_to_left(loc1: &Vec3, loc2: &Vec3) -> bool {
    (loc1.x - loc2.x) < 0.0
}
fn is_to_right(loc1: &Vec3, loc2: &Vec3) -> bool {
    (loc1.x - loc2.x) > 0.0 
}

fn abs_distance_x(loc1: &Vec3, loc2: &Vec3) -> f32 {
    f32::abs(loc1.x - loc2.x)
    // f32::abs(loc1.x - loc2.x) <= dist && f32::abs(loc1.y - loc2.y) <= clearance
}
fn abs_distance_y(loc1: &Vec3, loc2: &Vec3) -> f32 {
    f32::abs(loc1.y - loc2.y)
    // f32::abs(loc1.x - loc2.x) <= dist && f32::abs(loc1.y - loc2.y) <= clearance
}

fn ai(
    mut player_ship_query: Query<&Transform, With<Player>>,
    mut enemy_ship_query: Query<(&Transform, &mut Physics, &mut Weapon), With<Enemy>>,
) {
    if let Ok(player) = player_ship_query.single_mut() {
    
        for (transform, mut physics, mut weapon) in enemy_ship_query.iter_mut() {
            let mut thrust = Vec3::ZERO;
            if is_to_right(&player.translation, &transform.translation) {
                thrust += Vec3::X;
            }
            else if is_to_left(&player.translation, &transform.translation) {
                thrust -= Vec3::X;
            }
            physics.thrust = thrust;

            if abs_distance_x(&player.translation, &transform.translation) < 200.0 {
                weapon.fired = true;
            }
        }
    }
}






fn weapons(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    time: Res<Time>,
    materials: Res<Materials>,
    mut query: Query<(&Physics, &mut Weapon, &mut Transform)>,
) {
    for (ship_physics, mut weapon, transform) in query.iter_mut() {
        weapon.cooldown.tick(time.delta());
        if weapon.cooldown.finished() && weapon.fired {
            let bullet_speed_y = 750.0 * weapon.facing;
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.bullet.clone(),
                    transform: Transform {
                        translation: weapon.offset + transform.translation,
                        ..Default::default()
                    },
                    sprite: Sprite::new(Vec2::new(4.0, 10.0)),
                    ..Default::default()
                })
                .insert(Physics {
                    thrust: Vec3::ZERO,
                    max_speed: Vec3::new(2000.0, 500.0, 0.0),
                    acceleration: Vec3::ZERO,
                    velocity: Vec3::new(ship_physics.velocity.x / 2.0, bullet_speed_y, 0.0),
                    drag: Vec3::new(200.0, 200.0, 0.0)
                })
                .insert(Bullet {hit_damage: 10.0});
            weapon.fired = false;
            weapon.cooldown.reset();
        }
    }
}

fn location_despawn(
    mut commands: Commands,
    mut query: Query<(Entity, &Transform)>,
) {
    for (entity, location) in query.iter_mut() {
        if location.translation.y > 500. || location.translation.y < -500. {
            commands.entity(entity).despawn();
        }

    }
}

fn has_collided(loc1: &Vec3, loc2: &Vec3, dist: f32) -> bool {
    f32::abs(loc1.x - loc2.x) <= dist && f32::abs(loc1.y - loc2.y) <= dist
}

fn bullet_collisions(
    mut commands: Commands,
    mut wave: ResMut<Wave>,
    mut enemy_q: Query<(Entity, &mut Health, &mut Enemy, &mut Transform), Without<Bullet>>,
    mut bullet_q: Query<(Entity, &mut Bullet, &mut Transform), Without<Ship>>,
) {
    for (enemy_entity, mut health, _, enemy_transform) in enemy_q.iter_mut() {
        for (bullet_entity, bullet, bullet_transform) in bullet_q.iter_mut() {
            if has_collided(
                &enemy_transform.translation,
                &bullet_transform.translation,
                25., // TODO
            ) {
                health.level -= bullet.hit_damage;
                if health.level < 0.0 {
                    commands.entity(enemy_entity).despawn();
                    wave.enemies -= 1;
                }
                commands.entity(bullet_entity).despawn();
            }
        }
    }
}

fn animate_sprites(
    time: Res<Time>,
    texture_atlases: Res<Assets<TextureAtlas>>,
    mut query: Query<(&mut Timer, &mut TextureAtlasSprite, &Handle<TextureAtlas>)>,
) {
    for (mut timer, mut sprite, texture_atlas_handle) in query.iter_mut() {
        timer.tick(time.delta());
        if timer.finished() {
            let texture_atlas = texture_atlases.get(texture_atlas_handle).unwrap();
            sprite.index = ((sprite.index as usize + 1) % texture_atlas.textures.len()) as u32;
        }
    }
}