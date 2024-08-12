use crate::ecs::components::MainHandItemChanged;
use crate::inventory::Inventory;
use crate::main_hand::MainHand;
use specs::{Join, ReadStorage, System, WriteStorage};

pub struct UpdateMainHand;

impl<'a> System<'a> for UpdateMainHand {
    type SystemData = (
        WriteStorage<'a, MainHandItemChanged>,
        ReadStorage<'a, Inventory>,
        WriteStorage<'a, MainHand>,
    );

    fn run(&mut self, data: Self::SystemData) {
        let (mut main_hand_item_changed, inventory, mut main_hand) = data;

        for (_, inventory, main_hand) in
            (&main_hand_item_changed, &inventory, &mut main_hand).join()
        {
            main_hand.set_showing_item(inventory.get_selected_item());
        }

        main_hand_item_changed.clear();
    }
}
