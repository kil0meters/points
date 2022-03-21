#![windows_subsystem = "windows"]

use std::mem::swap;
use std::time::{Duration, Instant};

use bevy::app::Events;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::utils::HashMap;
use bevy::{prelude::*, utils::HashSet};
// use bevy_prototype_debug_lines::*;

use quadtree::{Circle, QuadTree};
use rand::Rng;

mod quadtree;

#[derive(Debug, Component)]
struct Point;

#[derive(Component, Clone, Copy)]
struct Velocity(Vec2);

#[derive(Component)]
struct WindowPosition {
    position: Vec2,
    time: Instant,
}

fn spawn_point(commands: &mut Commands, x: f32, y: f32) {
    let mut rng = rand::thread_rng();
    commands
        .spawn_bundle(SpriteBundle {
            transform: Transform {
                translation: Vec3::new(x, y, 0.0),
                scale: Vec3::new(5.0, 5.0, 5.0),
                ..Default::default()
            },
            sprite: Sprite {
                color: Color::hsl(
                    rng.gen_range(0.0..360.0),
                    0.7, //rng.gen_range(0.0..1.0),
                    0.5, //rng.gen_range(0.0..1.0),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(Point)
        .insert(Velocity(Vec2::new(
            rng.gen_range(-2.0..2.0),
            rng.gen_range(-2.0..2.0),
        )));
}

fn point_spawn_system(
    mut commands: Commands,
    mouse: Res<Input<MouseButton>>,
    windows: Res<Windows>,
) {
    if mouse.pressed(MouseButton::Left) {
        let win = windows.get_primary().unwrap();
        let pos = win.cursor_position().unwrap() - Vec2::new(win.width() / 2.0, win.height() / 2.0);

        spawn_point(&mut commands, pos.x, pos.y);
    }
}

fn point_movement_system(
    mut point_query: Query<(&Point, &Velocity, &mut Transform)>,
    windows: Res<Windows>,
) {
    let win = windows.get_primary().unwrap();
    let width = win.width() / 2.0;
    let height = win.height() / 2.0;

    for (_, velocity, mut transform) in point_query.iter_mut() {
        let translation = &mut transform.translation;
        translation.x += velocity.0.x;

        if translation.x > width {
            translation.x = -width;
        } else if translation.x < -width {
            translation.x = width;
        }

        translation.y += velocity.0.y;

        if translation.y > height {
            translation.y = -height;
        } else if translation.y < -height {
            translation.y = height;
        }
    }
}

fn window_moved_system(
    mut previous: ResMut<WindowPosition>,
    window_moved: Res<Events<WindowMoved>>,
    mut point_query: Query<(&Point, &mut Transform, &mut Velocity)>,
    time: Res<Time>,
) {
    let mut movement_reader = window_moved.get_reader();
    for event in movement_reader.iter(&window_moved) {
        let new_time = Instant::now();
        let new_position = Vec2::new(event.position.x as f32, event.position.y as f32);
        let position_change = new_position - previous.position;
        let change_velocity = position_change / (new_time - previous.time).as_secs_f32();

        for (_, mut transform, mut velocity) in point_query.iter_mut() {
            transform.translation.x -= position_change.x;
            transform.translation.y += position_change.y;

            velocity.0.x += change_velocity.x / 10000.0;
            velocity.0.y -= change_velocity.y / 10000.0;
        }

        previous.position = new_position;
        previous.time = new_time;
    }
    // previous_position.0 = (*window_moved).position;
}

fn point_collision_system(
    windows: Res<Windows>,
    // mut lines: ResMut<DebugLines>,
    mut point_query: Query<(Entity, &Point, &mut Velocity, &mut Transform)>,
) {
    let win = windows.get_primary().unwrap();
    let width = win.width();
    let height = win.height();
    let mut quadtree = QuadTree::new(-width / 2.0, -height / 2.0, width, height, 100);

    for (entity, _, _, transform) in point_query.iter_mut() {
        let translation = transform.translation;
        let pos = Vec2::new(translation.x, translation.y);

        quadtree.insert(pos, entity);
    }

    // quadtree.draw(&mut lines);

    let mut matching_points = HashSet::default();

    let mut collisions = HashMap::default();
    for (entity, _, velocity, transform) in point_query.iter() {
        let translation = transform.translation;

        quadtree.query(
            &Circle::new(translation.x, translation.y, 2.5),
            &mut matching_points,
        );

        for point in &matching_points {
            if point != &entity && !collisions.contains_key(&entity) {
                collisions.insert(*point, entity);
            }
        }

        matching_points.clear();
    }

    for (p1, p2) in collisions {
        // this is so fucking stupid i hate rust
        // println!("Collision between {:?} and {:?}", p1, p2);
        let v1 = *point_query.get_component::<Velocity>(p1).unwrap();

        let tmp = {
            let v2: &mut Velocity = &mut point_query.get_component_mut::<Velocity>(p2).unwrap();
            let tmp = v2.clone();
            v2.0 = -v1.0;
            tmp
        };

        let v1: &mut Velocity = &mut point_query.get_component_mut::<Velocity>(p1).unwrap();
        v1.0 = -tmp.0;
    }
}

fn main() {
    App::new()
        .insert_resource(WindowPosition {
            position: Vec2::new(0.0, 0.0),
            time: Instant::now(),
        })
        .add_plugins(DefaultPlugins)
        // .add_plugin(DebugLinesPlugin::default())
        .add_plugin(LogDiagnosticsPlugin::default())
        .insert_resource(ClearColor(Color::BLACK))
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_system(point_movement_system)
        .add_system(window_moved_system)
        .add_system(point_spawn_system)
        .add_system(point_collision_system)
        .add_startup_system(spawn_points)
        .add_startup_system(setup)
        .run();
}

fn spawn_points(mut commands: Commands) {
    let mut rng = rand::thread_rng();

    // for _ in 0..10000 {
    //     spawn_point(
    //         &mut commands,
    //         rng.gen_range(-640.0..640.0),
    //         rng.gen_range(-360.0..360.0),
    //     )
    // }
}

fn setup(mut commands: Commands) {
    commands.spawn_bundle(UiCameraBundle::default());
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}
