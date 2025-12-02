use bevy::prelude::*;

#[derive(Event, Debug, Clone, Copy)]
pub struct DamagePlayerEvent {
    pub target: Entity,
    pub amount: i32,
}

impl DamagePlayerEvent {
    pub fn new(target: Entity, amount: i32) -> DamagePlayerEvent {
        DamagePlayerEvent { target, amount }
    }
}

#[derive(Event)]
pub struct SanityDepletedEvent {
    pub player: Entity,
}