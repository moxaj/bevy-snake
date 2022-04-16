use bevy::core::FixedTimestep;
use bevy::prelude::*;

const ARENA_SIZE: u32 = 25;

const SNAKE_HEAD_SIZE: f32 = 0.8;
const SNAKE_HEAD_COLOR: Color = Color::rgb(0.8, 0.8, 0.8);

const SNAKE_SEGMENT_SIZE: f32 = 0.5;
const SNAKE_SEGMENT_COLOR: Color = Color::rgb(0.6, 0.6, 0.6);

const FOOD_SIZE: f32 = 0.6;
const FOOD_COLOR: Color = Color::rgb(0.2, 0.8, 0.2);

#[derive(Clone, Copy, PartialEq, Eq)]
enum Direction {
    Up,
    Right,
    Down,
    Left,
}

#[derive(Clone, Copy, Component)]
struct Size {
    width: f32,
    height: f32,
}

#[derive(Clone, Copy, PartialEq, Eq, Component)]
struct Position {
    x: i32,
    y: i32,
}

impl Position {
    fn do_move(&self, direction: Direction) -> Position {
        match direction {
            Direction::Up => Position {
                x: self.x,
                y: (self.y + 1).rem_euclid(ARENA_SIZE as i32),
            },
            Direction::Right => Position {
                x: (self.x + 1).rem_euclid(ARENA_SIZE as i32),
                y: self.y,
            },
            Direction::Down => Position {
                x: self.x,
                y: (self.y - 1).rem_euclid(ARENA_SIZE as i32),
            },
            Direction::Left => Position {
                x: (self.x - 1).rem_euclid(ARENA_SIZE as i32),
                y: self.y,
            },
        }
    }
}

#[derive(Component)]
struct SnakeSegment {
    next: Option<Entity>,
}

#[derive(Clone, Copy, Component)]
struct SnakeHead {
    direction: Direction,
    next_direction: Direction,
}

#[derive(Component)]
struct Food;

struct GrowEvent {
    position: Position,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
}

fn translate_position(windows: Res<Windows>, mut query: Query<(&Position, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    let window_size = window.width();
    let tile_size = window_size / ARENA_SIZE as f32;
    for (position, mut transform) in query.iter_mut() {
        transform.translation = Vec3::new(
            -window_size / 2. + tile_size / 2. + position.x as f32 * tile_size,
            -window_size / 2. + tile_size / 2. + position.y as f32 * tile_size,
            0.,
        );
    }
}

fn scale_size(windows: Res<Windows>, mut query: Query<(&Size, &mut Transform)>) {
    let window = windows.get_primary().unwrap();
    let window_size = window.width();
    let tile_size = window_size / ARENA_SIZE as f32;
    for (size, mut transform) in query.iter_mut() {
        transform.scale = Vec3::new(size.width * tile_size, size.height * tile_size, 1.);
    }
}

fn spawn_snake(mut commands: Commands) {
    let snake_tail1 = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Position { x: 14, y: 12 })
        .insert(Size {
            width: SNAKE_SEGMENT_SIZE,
            height: SNAKE_SEGMENT_SIZE,
        })
        .insert(SnakeSegment { next: None })
        .id();
    let snake_tail2 = commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_SEGMENT_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Position { x: 13, y: 12 })
        .insert(Size {
            width: SNAKE_SEGMENT_SIZE,
            height: SNAKE_SEGMENT_SIZE,
        })
        .insert(SnakeSegment {
            next: Some(snake_tail1),
        })
        .id();
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: SNAKE_HEAD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(Position { x: 12, y: 12 })
        .insert(Size {
            width: SNAKE_HEAD_SIZE,
            height: SNAKE_HEAD_SIZE,
        })
        .insert(SnakeHead {
            direction: Direction::Left,
            next_direction: Direction::Left,
        })
        .insert(SnakeSegment {
            next: Some(snake_tail2),
        });
}

fn handle_input(keyboard_input: Res<Input<KeyCode>>, mut snake_head_query: Query<&mut SnakeHead>) {
    let mut snake_head = snake_head_query.single_mut();
    if keyboard_input.pressed(KeyCode::Up) && snake_head.direction != Direction::Down {
        snake_head.next_direction = Direction::Up;
    } else if keyboard_input.pressed(KeyCode::Right) && snake_head.direction != Direction::Left {
        snake_head.next_direction = Direction::Right;
    } else if keyboard_input.pressed(KeyCode::Down) && snake_head.direction != Direction::Up {
        snake_head.next_direction = Direction::Down;
    } else if keyboard_input.pressed(KeyCode::Left) && snake_head.direction != Direction::Right {
        snake_head.next_direction = Direction::Left;
    }
}

fn move_snake(
    mut query_set: ParamSet<(
        Query<(Entity, &mut SnakeHead, &Position)>,
        Query<(&SnakeSegment, &mut Position)>,
    )>,
) {
    let mut snake_head_query = query_set.p0();
    let (mut snake_segment_entity, mut snake_head, snake_head_position) =
        snake_head_query.single_mut();
    snake_head.direction = snake_head.next_direction;
    let mut next_position = snake_head_position.do_move(snake_head.direction);

    let mut snake_segment_query = query_set.p1();
    loop {
        if let Ok((snake_segment, mut snake_segment_position)) =
            snake_segment_query.get_mut(snake_segment_entity)
        {
            let next_next_position = *snake_segment_position;
            snake_segment_position.x = next_position.x;
            snake_segment_position.y = next_position.y;
            next_position = next_next_position;
            if let Some(next_entity) = snake_segment.next {
                snake_segment_entity = next_entity;
            } else {
                break;
            }
        }
    }
}

fn spawn_food(
    mut commands: Commands,
    food_query: Query<&Position, With<Food>>,
    snake_segment_query: Query<&Position, With<SnakeSegment>>,
) {
    let food_position = loop {
        let food_position = Position {
            x: (rand::random::<f32>() * ARENA_SIZE as f32).floor() as i32,
            y: (rand::random::<f32>() * ARENA_SIZE as f32).floor() as i32,
        };
        if food_query
            .iter()
            .chain(snake_segment_query.iter())
            .all(|position| food_position != *position)
        {
            break food_position;
        }
    };
    commands
        .spawn_bundle(SpriteBundle {
            sprite: Sprite {
                color: FOOD_COLOR,
                ..default()
            },
            ..default()
        })
        .insert(food_position)
        .insert(Size {
            width: FOOD_SIZE,
            height: FOOD_SIZE,
        })
        .insert(Food);
}

fn eat_food(
    mut commands: Commands,
    food_query: Query<(Entity, &Position), With<Food>>,
    snake_head_query: Query<&Position, With<SnakeHead>>,
    snake_segment_query: Query<(&Position, &SnakeSegment)>,
    mut grow_event_writer: EventWriter<GrowEvent>,
) {
    let snake_head = snake_head_query.single();
    for (entity, food_position) in food_query.iter() {
        if *food_position == *snake_head {
            commands.entity(entity).despawn();
            grow_event_writer.send(GrowEvent {
                position: *snake_segment_query
                    .iter()
                    .find(|(_, snake_segment)| snake_segment.next.is_none())
                    .unwrap()
                    .0,
            });
        }
    }
}

fn grow_snake(
    mut commands: Commands,
    mut snake_segment_query: Query<(&Position, &mut SnakeSegment)>,
    mut event_reader: EventReader<GrowEvent>,
) {
    for grow_event in event_reader.iter() {
        snake_segment_query
            .iter_mut()
            .find(|(_, snake_segment)| snake_segment.next.is_none())
            .unwrap()
            .1
            .next = Some(
            commands
                .spawn_bundle(SpriteBundle {
                    sprite: Sprite {
                        color: SNAKE_SEGMENT_COLOR,
                        ..default()
                    },
                    ..default()
                })
                .insert(grow_event.position)
                .insert(Size {
                    width: SNAKE_SEGMENT_SIZE,
                    height: SNAKE_SEGMENT_SIZE,
                })
                .insert(SnakeSegment { next: None })
                .id(),
        );
    }
}

fn main() {
    App::new()
        .insert_resource(WindowDescriptor {
            title: "Snake".to_string(),
            width: 600.,
            height: 600.,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.04, 0.04, 0.04)))
        .add_event::<GrowEvent>()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup_camera)
        .add_startup_system(spawn_snake)
        .add_system(handle_input)
        .add_system(grow_snake.after(handle_input))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(0.08))
                .with_system(move_snake.after(grow_snake)),
        )
        .add_system(eat_food.after(move_snake))
        .add_system_set(
            SystemSet::new()
                .with_run_criteria(FixedTimestep::step(3.0))
                .with_system(spawn_food.after(move_snake)),
        )
        .add_system_set_to_stage(
            CoreStage::PostUpdate,
            SystemSet::new()
                .with_system(translate_position)
                .with_system(scale_size),
        )
        .run();
}
