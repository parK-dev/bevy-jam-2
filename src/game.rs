use std::time::Duration;
use bevy::prelude::*;
use iyes_loopless::prelude::*;

pub use assets::*;
pub use combining_system::*;
pub use components::*;
pub use spawn_item_system::*;

use crate::audio::record_player::animate;
use crate::audio::sound_event::SoundEvent;
use crate::game::camera::create_camera;
use crate::game::create_grid_system::create_grids;
use crate::game::dragging::{
    apply_scrim_to_being_dragged, check_drag_begin, check_drag_end, check_ghost_placement_validity,
    process_drag_event, set_ghost_position, DragEvent,
};
use crate::hud::gold::{gold_update_system, setup_gold};
use crate::mouse::{reset_cursor, set_cursor_appearance, Mouse};
use crate::AppState;
use crate::game::items::{Item, ItemId};
use crate::positioning::{Coords, Dimens, Pos};

use self::items::CraftItem;
use crate::game::dungeonsim::{dungeon_text_test, DungeonState, init_dungeon};

pub mod assets;
pub mod camera;
mod combining_system;
mod components;
mod create_grid_system;
pub mod dragging;
mod dragging;
mod dungeonsim;
pub mod items;
pub mod recipes;
mod spawn_item_system;

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<SpawnItemEvent>()
            .add_event::<DragEvent>()
            .init_resource::<Player>()
            .insert_resource(DungeonState{msg_cooldown: Timer::new(Duration::from_millis(3000), true), running: true })
            .add_enter_system_set(
                AppState::InGame,
                ConditionSet::new()
                    .run_in_state(AppState::InGame)
                    .with_system(setup)
                    .with_system(setup_gold)
                    .with_system(setup_spawn_item_timer)
                    .with_system(create_camera)
                    .with_system(create_grids)
                    .with_system(configure_cursor)
                    .with_system(init_dungeon)
                    .with_system(create_debug_items)
                    .into(),
            )
            .add_system_set(
                ConditionSet::new()
                    .run_in_state(AppState::InGame)
                    // .with_system(spawn_item_timer_system)
                    .with_system(spawn_item)
                    .with_system(set_cursor_appearance)
                    .with_system(check_drag_begin)
                    .with_system(set_ghost_position)
                    .with_system(apply_scrim_to_being_dragged)
                    .with_system(check_ghost_placement_validity)
                    .with_system(check_drag_end)
                    .with_system(process_drag_event)
                    .with_system(combine_items_system)
                    .with_system(gold_update_system)
                    .with_system(dungeon_text_test)
                    .with_system(animate)
                    .with_system(track_combine_button_hover)
                    .into(),
            )
            .add_exit_system_set(
                AppState::InGame,
                ConditionSet::new()
                    .run_in_state(AppState::InGame)
                    .with_system(despawn_gameplay_entities)
                    .with_system(reset_cursor)
                    .into(),
            );
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Component)]
pub enum GameResult {
    Lost,
    Won,
}

// TODO: Move this to it's own system?
fn setup(mut audio: EventWriter<SoundEvent>) {
    audio.send(SoundEvent::Music(Some((MusicId::Placeholder, false))));
}

pub fn despawn_gameplay_entities(
    mut cmd: Commands,
    mut audio: EventWriter<SoundEvent>,
    q: Query<Entity, With<CleanupOnGameplayEnd>>,
) {
    for e in q.iter() {
        cmd.entity(e).despawn_recursive();
    }
    audio.send(SoundEvent::KillAllMusic);
}

// This feels overkill, with a set window size we could use regular UI entities instead that
// come with the interactions component premade
pub fn track_combine_button_hover(
    mut audio: EventWriter<SoundEvent>,
    input: Res<Input<MouseButton>>,
    query_mouse: Query<&Mouse>,
    mut button: Query<(&mut Sprite, &Transform, &CombineButton)>,
) {
    let mouse = query_mouse.single();
    let mouse_hovers_over_button = button.get_single().map_or(false, |(_, transform, button)| {
        mouse.position.x > transform.translation.x - button.coords.dimens.x as f32 * 0.5
            && mouse.position.x < transform.translation.x + button.coords.dimens.x as f32 * 0.5
            && mouse.position.y > transform.translation.y - button.coords.dimens.y as f32 * 0.5
            && mouse.position.y < transform.translation.y + button.coords.dimens.y as f32 * 0.5
    });

    if mouse_hovers_over_button && input.just_pressed(MouseButton::Left) {
        audio.send(SoundEvent::Sfx(SoundId::Placeholder));
        if let Ok((mut sprite, _, _)) = button.get_single_mut() {
            if mouse_hovers_over_button {
                sprite.color = Color::rgba(255.0, 255.0, 255.0, 0.8);
            }
        }
        // TODO: Check is_valid_recipe with craft_items, combine()
    } else if let Ok((mut sprite, _, _)) = button.get_single_mut() {
        sprite.color = Color::rgba(0.2, 0.2, 0.2, 0.8);
    }
}

pub fn create_debug_items(mut spawn: EventWriter<SpawnItemEvent>) {
    spawn.send(SpawnItemEvent::new(
        Item {
            id: ItemId::CandleStick,
            texture_id: TextureId::CandleStick,
            name: "".to_string(),
            description: "".to_string(),
            wearable: None,
        },
        Coords::new(Pos::new(10, 10), Dimens::new(1, 2)),
    ));
    spawn.send(SpawnItemEvent::new(
        Item {
            id: ItemId::EmptyLantern,
            texture_id: TextureId::EmptyLantern,
            name: "".to_string(),
            description: "".to_string(),
            wearable: None,
        },
        Coords::new(Pos::new(5, 5), Dimens::new(2, 3)),
    ));
    spawn.send(SpawnItemEvent::new(
        Item {
            id: ItemId::FireEssence,
            texture_id: TextureId::FireEssence,
            name: "".to_string(),
            description: "".to_string(),
            wearable: None,
        },
        Coords::new(Pos::new(15, 15), Dimens::new(1, 1)),
    ));
    spawn.send(SpawnItemEvent::new(
        Item {
            id: ItemId::Vial,
            texture_id: TextureId::Vial,
            name: "".to_string(),
            description: "".to_string(),
            wearable: None,
        },
        Coords::new(Pos::new(20, 20), Dimens::new(3, 2)),
    ));
    spawn.send(SpawnItemEvent::new(
        Item {
            id: ItemId::TurtleHerb,
            texture_id: TextureId::TurtleHerb,
            name: "".to_string(),
            description: "".to_string(),
            wearable: None,
        },
        Coords::new(Pos::new(25, 25), Dimens::new(2, 3)),
    ));
}
