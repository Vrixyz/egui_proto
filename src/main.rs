mod utils;

use bevy::{audio::CpalSample, gizmos, prelude::*, transform};
use bevy_egui::{
    egui::{self, Align2},
    EguiContexts, EguiPlugin,
};
use bevy_pancam::{PanCam, PanCamPlugin};

#[derive(Debug, States, Reflect, Hash, PartialEq, Eq, Clone)]
pub enum InteractionState {
    Editing,
    Visualizing,
}

#[derive(Debug, Component, Reflect)]
pub struct Destination {
    pub location: Vec2,
}
#[derive(Debug, Component, Reflect)]
pub struct Position(Vec2);

#[derive(Debug, Component, Reflect)]
pub struct Health(f32);

#[derive(Debug, Component, Reflect)]
pub struct Speed(f32);

#[derive(Debug, Component, Reflect)]
pub struct Editing(bool);

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, PanCamPlugin))
        .add_plugins(EguiPlugin)
        .insert_state(InteractionState::Editing)
        // Systems that create Egui widgets should be run during the `CoreSet::Update` set,
        // or after the `EguiSet::BeginFrame` system (which belongs to the `CoreSet::PreUpdate` set).
        .add_systems(Update, ui_edit.run_if(in_state(InteractionState::Editing)))
        .add_systems(Update, render)
        .add_systems(
            Update,
            ui_menu_editing.run_if(in_state(InteractionState::Editing)),
        )
        .add_systems(
            Update,
            ui_menu_visualizing.run_if(in_state(InteractionState::Visualizing)),
        )
        .add_systems(
            Update,
            update_visualization.run_if(in_state(InteractionState::Visualizing)),
        )
        .add_systems(Startup, (base_setup, create_units))
        .run();
}

fn base_setup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(PanCam::default());
}

fn create_units(mut commands: Commands) {
    let mut to_spawn = vec![];
    for i in 0..50 {
        let position = Vec2::new(i as f32 * 50f32, i as f32 * 50f32);
        to_spawn.push((
            Position(position),
            Destination { location: position },
            Health(1f32),
            Editing(false),
            Speed(50f32),
        ));
    }
    commands.spawn_batch(to_spawn);
}

fn render(mut gizmos: Gizmos, q_to_show: Query<&Position>) {
    for p in q_to_show.iter() {
        gizmos.circle_2d(p.0, 50f32, Color::RED);
    }
}
fn update_visualization(
    mut q_moving: Query<(&Destination, &mut Position, &Speed)>,
    time: Res<Time>,
) {
    for (destination, mut position, speed) in q_moving.iter_mut() {
        if destination.location != position.0 {
            position.0 = utils::move_towards(
                position.0,
                destination.location,
                speed.0 * time.delta_seconds(),
            )
        }
    }
}

fn ui_edit(
    mut contexts: EguiContexts,
    q_camera: Query<(&Camera, &GlobalTransform)>,
    mut q_to_edit: Query<(Entity, &mut Editing, &mut Destination, &Position)>,
) {
    for (entity, mut editing, mut destination, position) in q_to_edit.iter_mut() {
        let (cam, g_transform) = q_camera.single() else {
            return;
        };
        let Some(viewport_position) = cam.world_to_viewport(g_transform, position.0.extend(-30f32))
        else {
            return;
        };
        egui::Area::new(format!("unit {}", entity.index()))
            .fixed_pos(egui::pos2(0f32, 0f32))
            .anchor(
                Align2::LEFT_TOP,
                egui::vec2(viewport_position.x, viewport_position.y),
            )
            .show(contexts.ctx_mut(), |ui| {
                if ui.button(if editing.0 { "hide" } else { "edit" }).clicked() {
                    editing.0 = !editing.0;
                }
                if editing.0 {
                    let mut text_x = destination.location.x.to_string();
                    if ui.text_edit_singleline(&mut text_x).changed() {
                        if let Ok(new_x) = text_x.parse::<f32>() {
                            destination.location.x = new_x;
                        }
                    }
                    let mut text_y = destination.location.y.to_string();

                    if ui.text_edit_singleline(&mut text_y).changed() {
                        if let Ok(new_y) = text_y.parse::<f32>() {
                            destination.location.y = new_y;
                        }
                    }
                }
            });
    }
}
fn ui_menu_editing(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<InteractionState>>,
) {
    egui::SidePanel::left("menu_editing").show(contexts.ctx_mut(), |ui| {
        if ui.button("Run simulation").clicked() {
            next_state.set(InteractionState::Visualizing);
        }
    });
}
fn ui_menu_visualizing(
    mut contexts: EguiContexts,
    mut next_state: ResMut<NextState<InteractionState>>,
) {
    egui::SidePanel::left("menu_visualizing").show(contexts.ctx_mut(), |ui| {
        if ui.button("Go to edit").clicked() {
            next_state.set(InteractionState::Editing);
        }
    });
}
