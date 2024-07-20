pub mod item;

use crate::chunk::BlockID;
use crate::inventory::item::ItemStack;
use crate::types::UVMap;
use glfw::WindowEvent;

const INVENTORY_SIZE: usize = 36;
const HOTBAR_SIZE: usize = 9;

pub struct Inventory {
    pub slots: [Option<ItemStack>; INVENTORY_SIZE],
    pub selected_hotbar_slot: usize,
}

impl Inventory {
    pub fn new(uv_map: &UVMap) -> Self {
        Self {
            slots: {
                let mut slots = [None; INVENTORY_SIZE];
                slots[0] = Some(ItemStack::new(BlockID::Dirt, 1, uv_map));
                slots[1] = Some(ItemStack::new(BlockID::GrassBlock, 1, uv_map));
                slots[2] = Some(ItemStack::new(BlockID::Cobblestone, 1, uv_map));
                slots[3] = Some(ItemStack::new(BlockID::OakLog, 1, uv_map));
                slots[4] = Some(ItemStack::new(BlockID::OakPlanks, 1, uv_map));
                slots[5] = Some(ItemStack::new(BlockID::OakLeaves, 1, uv_map));
                slots[6] = Some(ItemStack::new(BlockID::Glass, 1, uv_map));
                slots[7] = Some(ItemStack::new(BlockID::Obsidian, 1, uv_map));
                slots[8] = Some(ItemStack::new(BlockID::Debug, 1, uv_map));

                slots
            },
            selected_hotbar_slot: 0,
        }
    }

    pub fn get_selected_item(&self) -> Option<BlockID> {
        self.slots[self.selected_hotbar_slot].map(|item_stack| item_stack.item)
    }

    pub fn handle_input_event(&mut self, event: &WindowEvent) {
        use glfw::{Action, Key};

        match event {
            WindowEvent::Scroll(_, y) => {
                if y.is_sign_positive() {
                    self.select_next_item();
                } else {
                    self.select_prev_item();
                }
            }
            WindowEvent::Key(Key::Num1, _, Action::Press, _) => self.select_item(0),
            WindowEvent::Key(Key::Num2, _, Action::Press, _) => self.select_item(1),
            WindowEvent::Key(Key::Num3, _, Action::Press, _) => self.select_item(2),
            WindowEvent::Key(Key::Num4, _, Action::Press, _) => self.select_item(3),
            WindowEvent::Key(Key::Num5, _, Action::Press, _) => self.select_item(4),
            WindowEvent::Key(Key::Num6, _, Action::Press, _) => self.select_item(5),
            WindowEvent::Key(Key::Num7, _, Action::Press, _) => self.select_item(6),
            WindowEvent::Key(Key::Num8, _, Action::Press, _) => self.select_item(7),
            WindowEvent::Key(Key::Num9, _, Action::Press, _) => self.select_item(8),
            _ => {}
        }
    }

    pub fn select_item(&mut self, index: usize) {
        self.selected_hotbar_slot = index;
    }

    pub fn select_next_item(&mut self) {
        self.selected_hotbar_slot += 1;

        if self.selected_hotbar_slot >= HOTBAR_SIZE {
            self.selected_hotbar_slot = 0;
        }
    }

    pub fn select_prev_item(&mut self) {
        if self.selected_hotbar_slot == 0 {
            self.selected_hotbar_slot = HOTBAR_SIZE - 1;
        } else {
            self.selected_hotbar_slot -= 1;
        }
    }
}
