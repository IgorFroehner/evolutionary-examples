use std::time::Duration;

use bevy::{prelude::*, time::common_conditions::on_timer};
use evolutionary::prelude::*;
use maze::read_matrix_from_file;
use maze_fitness::MazeFitness;

mod maze;
mod maze_fitness;

#[derive(Clone)]
pub struct MazeCoding;

impl Coding<Real> for MazeCoding {
    type Output = ();

    fn decode(&self, _individual: &Real) -> Self::Output {
        ()
    }
}

#[derive(Resource)]
pub struct EvolutionResource {
    evolution: Evolution<Real, MazeCoding>,
}

#[derive(Resource)]
pub struct Maze {
    maze: Vec<Vec<i32>>,
    start: (usize, usize),
    end: (usize, usize),
}

#[derive(Component)]
pub struct PathCell;

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq, Hash, States)]
pub enum EvolutionStates {
    #[default]
    Running,
    Finished,
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

fn setup(
    mut commands: Commands,
    windows: Query<&mut Window>,
    mut evolution: ResMut<EvolutionResource>,
    maze: Res<Maze>,
) {
    evolution.evolution.start();

    let window = windows.single();

    let width = window.resolution.width() as usize;
    let height = window.resolution.height() as usize;

    let n = maze.maze.len();
    let m = maze.maze[0].len();

    let cell_size = Vec2::new(width as f32 / m as f32, height as f32 / n as f32);
    let first_position_x = cell_size.x / 2.0 - width as f32 / 2.0;
    let first_position_y = height as f32 / 2.0 - cell_size.y / 2.0;

    for i in 0..n {
        for j in 0..m {
            let color = if maze.maze[i][j] == 0 {
                Color::BLACK
            } else if maze.maze[i][j] == 1 {
                Color::WHITE
            } else if maze.maze[i][j] == 2 {
                Color::GREEN
            } else {
                Color::BLUE
            };

            let xcell_position = first_position_x + j as f32 * cell_size.x;
            let ycell_position = first_position_y - i as f32 * cell_size.y;

            commands.spawn(SpriteBundle {
                transform: Transform::from_xyz(xcell_position, ycell_position, 1.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(cell_size.x - 1.0, cell_size.y - 1.0)),
                    color: color,
                    ..Default::default()
                },
                ..default()
            });
        }
    }
}

fn update(
    mut commands: Commands,
    mut next_state: ResMut<NextState<EvolutionStates>>,
    windows: Query<&mut Window>,
    query: Query<Entity, With<PathCell>>,
    maze: Res<Maze>,
    mut evolution: ResMut<EvolutionResource>,
) {
    evolution.evolution.next();

    if evolution.evolution.reached_stop_condition() {
        println!("Finished");
        next_state.set(EvolutionStates::Finished);
    }

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    let window = windows.single();

    let width = window.resolution.width() as usize;
    let height = window.resolution.height() as usize;

    let n = maze.maze.len();
    let m = maze.maze[0].len();

    let cell_size = Vec2::new(width as f32 / m as f32, height as f32 / n as f32);
    let first_position_x = cell_size.x / 2.0 - width as f32 / 2.0;
    let first_position_y = height as f32 / 2.0 - cell_size.y / 2.0;

    let population = evolution.evolution.current_population();

    for individual in population {
        let chromossome = individual.chromosome;

        let path = MazeFitness::calculate_path(&chromossome, &maze.maze, maze.start);

        let final_position = path.last().unwrap();

        let (i, j) = final_position;

        let xcell_position = first_position_x + *j as f32 * cell_size.x;
        let ycell_position = first_position_y - *i as f32 * cell_size.y;

        commands.spawn((
            PathCell,
            SpriteBundle {
                transform: Transform::from_xyz(xcell_position, ycell_position, 3.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(cell_size.x - 1.0, cell_size.y - 1.0)),
                    color: Color::YELLOW,
                    ..Default::default()
                },
                ..default()
            },
        ));
    }

    let best_individual = evolution.evolution.current_best();

    let path = MazeFitness::calculate_path(&best_individual.chromosome, &maze.maze, maze.start);

    for a in path {
        let (i, j) = a;

        let xcell_position = first_position_x + j as f32 * cell_size.x;
        let ycell_position = first_position_y - i as f32 * cell_size.y;

        commands.spawn((
            PathCell,
            SpriteBundle {
                transform: Transform::from_xyz(xcell_position, ycell_position, 2.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(cell_size.x - 1.0, cell_size.y - 1.0)),
                    color: Color::rgba_linear(0.9, 0.9, 0.1, 0.6),
                    ..Default::default()
                },
                ..default()
            },
        ));
    }
}

fn finished(
    mut commands: Commands,
    windows: Query<&mut Window>,
    query: Query<Entity, With<PathCell>>,
    maze: Res<Maze>,
    mut evolution: ResMut<EvolutionResource>,
) {
    evolution.evolution.next();

    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    let window = windows.single();

    let width = window.resolution.width() as usize;
    let height = window.resolution.height() as usize;

    let n = maze.maze.len();
    let m = maze.maze[0].len();

    let cell_size = Vec2::new(width as f32 / m as f32, height as f32 / n as f32);
    let first_position_x = cell_size.x / 2.0 - width as f32 / 2.0;
    let first_position_y = height as f32 / 2.0 - cell_size.y / 2.0;

    let best_individual = evolution.evolution.current_best();

    let mut path = MazeFitness::calculate_path(&best_individual.chromosome, &maze.maze, maze.start);

    println!("Path size: {}", path.len());
    path.pop().unwrap();

    for a in path {
        let (i, j) = a;

        let xcell_position = first_position_x + j as f32 * cell_size.x;
        let ycell_position = first_position_y - i as f32 * cell_size.y;

        commands.spawn((
            PathCell,
            SpriteBundle {
                transform: Transform::from_xyz(xcell_position, ycell_position, 2.0),
                sprite: Sprite {
                    custom_size: Some(Vec2::new(cell_size.x - 1.0, cell_size.y - 1.0)),
                    color: Color::rgba_linear(0.1, 0.8, 0.2, 0.9),
                    ..Default::default()
                },
                ..default()
            },
        ));
    }
}

fn main() {
    let mut maze = read_matrix_from_file("examples/maze/maze4.in");

    let mut start = (0, 0);
    let mut end = (0, 0);
    for i in 0..maze.len() {
        for j in 0..maze[0].len() {
            if maze[i][j] == 2 {
                start = (i, j);
            }
            if maze[i][j] == 3 {
                end = (i, j);
            }
        }
    }

    let n = maze.len();
    let m = maze[0].len();

    let max_score = (n + m) as f64;

    let crossover = UniformCrossover {
        crossover_rate: 0.2,
        toss_probability: 0.3,
    };

    let mutation = SubstituteMutation {
        mutation_rate: 0.1,
    };

    let evolution_builder = EvolutionBuilder::new(50, 2000, GeneCod::Real, (0.0, 1.0))
        .with_fitness(MazeFitness {
            max_dist: max_score,
            start,
            end,
            maze: maze.clone(),
        })
        .with_selection(RouletteSelection::default())
        .with_crossover(crossover)
        .with_mutation(mutation)
        .with_title("Maze".to_string())
        .with_stop_condition(move |best_fitness, _, _| best_fitness == max_score)
        // .with_stop_condition(move |_, iterations, _| iterations >= 10_000)
        .with_coding(MazeCoding);

    let evolution = evolution_builder.build().unwrap();

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Maze".into(),
                resolution: (1080.0, 720.0).into(),
                resizable: false,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .insert_resource(Maze { maze, start, end })
        .insert_resource(EvolutionResource { evolution })
        .add_state::<EvolutionStates>()
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, setup)
        .add_systems(
            Update,
            update
                .run_if(in_state(EvolutionStates::Running))
                // .run_if(on_timer(Duration::from_millis(100))),
        )
        .add_systems(OnEnter(EvolutionStates::Finished), finished)
        .run();
}
