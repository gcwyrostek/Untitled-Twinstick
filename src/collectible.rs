use crate::{GameState, components::Health, player::Player};
use bevy::prelude::*;

#[derive(Component)]
pub struct Collectible {
    pub collectible_type: CollectibleType,
    pub amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CollectibleType {
    ReviveKit,
    Ammo(i32),    // Amount of ammo
    Battery(i32), // Amount of battery power
    Health(i32),  // Amount of health
    Flashlight,   // flashlight
}

#[derive(Resource)]
pub struct PlayerInventory {
    pub revive_kits: i32,
    pub magazine: i32,      // Change this to adjust the current magazine size.
    pub reserve: i32,       // Change this to adjust the current reserve size.
    pub max_revive_kits: i32,
    pub max_magazine: i32,  // Change this to set the magazine capacity.
    pub max_reserve: i32,   // Change this to set the reserve capacity.
    pub has_flashlight: bool,
}

impl Default for PlayerInventory {
    fn default() -> Self {
        Self {
            revive_kits: 0,
            magazine: 30,     // Starting magazine size.
            reserve: 60,      // Starting reserve size.
            max_revive_kits: 1,
            max_magazine: 30, // Default magazine capacity.
            max_reserve: 180, // Default reserve capacity.
            has_flashlight: false,
        }
    }
}

impl PlayerInventory {
    pub fn has_available_ammo(&self) -> bool {
        self.magazine > 0 || self.reserve > 0
    }

    pub fn reload(&mut self) -> bool {
        if self.reserve <= 0 || self.magazine >= self.max_magazine {
            return false;
        }

        let needed = self.max_magazine - self.magazine;
        let to_load = needed.min(self.reserve);
        self.magazine += to_load;
        self.reserve -= to_load;
        to_load > 0
    }

    pub fn ensure_magazine_ready(&mut self) {
        if self.magazine == 0 {
            self.reload();
        }
    }

    pub fn consume_rounds(&mut self, rounds: i32) -> bool {
        if rounds <= 0 {
            return true;
        }

        let mut remaining = rounds;

        while remaining > 0 {
            if self.magazine == 0 {
                if !self.reload() {
                    return false;
                }
            }

            self.magazine -= 1;
            remaining -= 1;
        }

        true
    }

    pub fn add_to_reserve(&mut self, rounds: i32) -> i32 {
        if rounds <= 0 {
            return 0;
        }

        let space = (self.max_reserve - self.reserve).max(0);
        let added = rounds.min(space);
        self.reserve += added;

        // Auto reload if magazine is empty and reserve now has ammo
        if self.magazine == 0 {
            self.ensure_magazine_ready();
        }

        added
    }
}

pub struct CollectiblePlugin;

impl Plugin for CollectiblePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInventory>()
            .add_systems(OnEnter(GameState::Playing), setup_collectibles);
    }
}

pub fn setup_collectibles(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Spawn some revive kits
    /*for i in 0..3 {
        commands.spawn((
            Sprite::from_image(asset_server.load("revive kit/Revive Kit_albedo.png")),
            Transform::from_xyz(200.0 + (i as f32 * 150.0), 300.0, 5.0) // Exact location, could be replaced with random in the future
                .with_scale(Vec3::splat(0.5)), // 0.5x smaller than original size
            Collectible {
                collectible_type: CollectibleType::ReviveKit, // Revive kit type
                amount: 1,                                    // Revive kit amount
            },
        ));
    }*/

    // Spawn some bullet pickups (ammo)
    for i in 0..3 {
        // Change the range upper bound to control how many bullet pickups spawn.
        commands.spawn((
            Sprite::from_image(asset_server.load("textures/bullet.png")),
            Transform::from_xyz(-200.0 + (i as f32 * 100.0), -200.0, 5.0)
                .with_scale(Vec3::splat(0.6)),
            Collectible {
                collectible_type: CollectibleType::Ammo(30),
                amount: 30,
            },
        ));
    }

    // Spawn some health pickups
    for i in 0..4 {
        commands.spawn((
            Sprite::from_image(asset_server.load("textures/health_pickup.png")),
            Transform::from_xyz(100.0 + (i as f32 * 120.0), -300.0, 5.0) // Exact location, could be replaced with random in the future
                .with_scale(Vec3::splat(2.0)), // 2x larger than original size
            Collectible {
                collectible_type: CollectibleType::Health(10), // Health amount
                amount: 10,                                    // Health amount
            },
        ));
    }

    // Spawn some battery pickups
    for i in 0..3 {
        commands.spawn((
            Sprite::from_image(asset_server.load("textures/battery.png")),
            Transform::from_xyz(-300.0 + (i as f32 * 150.0), 100.0, 5.0)
                .with_scale(Vec3::splat(1.8)), // 1.8x larger than original size
            Collectible {
                collectible_type: CollectibleType::Battery(10), // Battery power amount
                amount: 10,                                     // Battery power amount
            },
        ));
    }

    // Spawn some flashlight pickups
    for i in 0..2 {
        commands.spawn((
            Sprite::from_image(asset_server.load("textures/flashlight.png")),
            Transform::from_xyz(50.0 + (i as f32 * 180.0), 200.0, 5.0).with_scale(Vec3::splat(1.2)), // 1.2x larger than original size
            Collectible {
                collectible_type: CollectibleType::Flashlight, // Flashlight pickup (boolean to check if picked up or not)
                amount: 1,                                     // Single flashlight
            },
        ));
    }
}

// Helper functions for future use

// Helper function to use revive kit on dead player
pub fn use_revive_kit(
    inventory: &mut ResMut<PlayerInventory>,
    dead_player_entity: Entity,
    players: &mut Query<&mut Health, With<Player>>,
) -> bool {
    if inventory.revive_kits > 0 {
        if let Ok(mut health) = players.get_mut(dead_player_entity) {
            health.current = health.max; // Full heal
            inventory.revive_kits -= 1;
            println!("Used revive kit! Player revived with full health.");
            return true;
        }
    }
    false
}

// Helper function to consume ammo
pub fn consume_ammo(inventory: &mut ResMut<PlayerInventory>, amount: i32) -> bool {
    inventory.consume_rounds(amount)
}

// Helper function to check if player can shoot
pub fn can_shoot(inventory: &Res<PlayerInventory>) -> bool {
    inventory.has_available_ammo()
}

// Helper function to grant flashlight
pub fn pickup_flashlight(inventory: &mut ResMut<PlayerInventory>) {
    if !inventory.has_flashlight {
        inventory.has_flashlight = true;
        println!("Picked up flashlight!");
    }
}
