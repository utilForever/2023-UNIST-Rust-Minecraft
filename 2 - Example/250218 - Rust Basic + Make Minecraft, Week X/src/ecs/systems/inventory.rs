use crate::chunk_manager::ChunkManager;
use crate::ecs::components::MainHandItemChanged;
use crate::input::InputCache;
use crate::inventory::item::ItemStack;
use crate::inventory::Inventory;
use crate::player::PlayerState;
use glfw::{Action, Key, MouseButton, WindowEvent};
use specs::{Entities, Join, Read, ReadStorage, System, WriteStorage};
use std::sync::Arc;

pub struct InventoryHandleInput;

impl InventoryHandleInput {
    fn select_item(inventory: &mut Inventory, index: usize, f: &mut dyn FnMut()) {
        if inventory.selected_hotbar_slot != index {
            inventory.select_item(index);
            f();
        }
    }
}

impl<'a> System<'a> for InventoryHandleInput {
    type SystemData = (
        Entities<'a>,
        Read<'a, InputCache>,
        Read<'a, Arc<ChunkManager>>,
        ReadStorage<'a, PlayerState>,
        WriteStorage<'a, Inventory>,
        WriteStorage<'a, MainHandItemChanged>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (
            entities,
            input_cache,
            chunk_manager,
            player_state,
            mut inventory,
            mut main_hand_item_changed,
        ) = data;

        for (entity, inventory, player_state) in (&entities, &mut inventory, &player_state).join() {
            let mut f = || {
                if let Err(e) = main_hand_item_changed.insert(entity, MainHandItemChanged) {
                    error!("{e}");
                }
            };

            for event in input_cache.events.iter() {
                match event {
                    WindowEvent::Scroll(_, y) => {
                        if y.is_sign_positive() {
                            inventory.select_next_item();
                        } else {
                            inventory.select_prev_item();
                        }

                        f();
                    }
                    WindowEvent::MouseButton(MouseButton::Button3, Action::Press, _) => {
                        if let Some(((x, y, z), _)) = player_state.targeted_block {
                            if let Some(block) = chunk_manager.get_block(x, y, z) {
                                inventory.slots[inventory.selected_hotbar_slot] =
                                    Some(ItemStack::new(block, 1));
                                f();
                            }
                        }
                    }
                    WindowEvent::Key(Key::Num1, _, Action::Press, _) => {
                        Self::select_item(inventory, 0, &mut f)
                    }
                    WindowEvent::Key(Key::Num2, _, Action::Press, _) => {
                        Self::select_item(inventory, 1, &mut f)
                    }
                    WindowEvent::Key(Key::Num3, _, Action::Press, _) => {
                        Self::select_item(inventory, 2, &mut f)
                    }
                    WindowEvent::Key(Key::Num4, _, Action::Press, _) => {
                        Self::select_item(inventory, 3, &mut f)
                    }
                    WindowEvent::Key(Key::Num5, _, Action::Press, _) => {
                        Self::select_item(inventory, 4, &mut f)
                    }
                    WindowEvent::Key(Key::Num6, _, Action::Press, _) => {
                        Self::select_item(inventory, 5, &mut f)
                    }
                    WindowEvent::Key(Key::Num7, _, Action::Press, _) => {
                        Self::select_item(inventory, 6, &mut f)
                    }
                    WindowEvent::Key(Key::Num8, _, Action::Press, _) => {
                        Self::select_item(inventory, 7, &mut f)
                    }
                    WindowEvent::Key(Key::Num9, _, Action::Press, _) => {
                        Self::select_item(inventory, 8, &mut f)
                    }
                    _ => {}
                }
            }
        }
    }
}
