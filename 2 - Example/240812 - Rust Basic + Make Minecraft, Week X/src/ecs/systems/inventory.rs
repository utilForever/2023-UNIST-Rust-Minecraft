use crate::ecs::components::MainHandItemChanged;
use crate::input::InputCache;
use crate::inventory::Inventory;
use glfw::{Action, Key, WindowEvent};
use specs::{Entities, Join, Read, System, WriteStorage};

pub struct InventoryHandleInput;

impl<'a> System<'a> for InventoryHandleInput {
    type SystemData = (
        Entities<'a>,
        Read<'a, InputCache>,
        WriteStorage<'a, Inventory>,
        WriteStorage<'a, MainHandItemChanged>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (entities, input_cache, mut inventory, mut main_hand_item_changed) = data;

        for (entity, inventory) in (&entities, &mut inventory).join() {
            let mut f = || {
                main_hand_item_changed
                    .insert(entity, MainHandItemChanged)
                    .expect("Cannot insert MainHandItemChanged");
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
                    WindowEvent::Key(Key::Num1, _, Action::Press, _) => {
                        inventory.select_item(0);
                        f();
                    }
                    WindowEvent::Key(Key::Num2, _, Action::Press, _) => {
                        inventory.select_item(1);
                        f();
                    }
                    WindowEvent::Key(Key::Num3, _, Action::Press, _) => {
                        inventory.select_item(2);
                        f();
                    }
                    WindowEvent::Key(Key::Num4, _, Action::Press, _) => {
                        inventory.select_item(3);
                        f();
                    }
                    WindowEvent::Key(Key::Num5, _, Action::Press, _) => {
                        inventory.select_item(4);
                        f();
                    }
                    WindowEvent::Key(Key::Num6, _, Action::Press, _) => {
                        inventory.select_item(5);
                        f();
                    }
                    WindowEvent::Key(Key::Num7, _, Action::Press, _) => {
                        inventory.select_item(6);
                        f();
                    }
                    WindowEvent::Key(Key::Num8, _, Action::Press, _) => {
                        inventory.select_item(7);
                        f();
                    }
                    WindowEvent::Key(Key::Num9, _, Action::Press, _) => {
                        inventory.select_item(8);
                        f();
                    }
                    _ => {}
                }
            }
        }
    }
}
