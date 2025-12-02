use bevy::prelude::*;
use crate::{
    net_control::{NetControl, PlayerType},
    collectible::CollectibleType::Flashlight,
    player::Player,
    components::Sanity,
    events::DamagePlayerEvent,
};

pub struct SanityPlugin;

impl Plugin for SanityPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SanityTimers>()
            .add_event::<DamagePlayerEvent>()
            .add_systems(Update, sanity_drain_system)
            .add_systems(Update, sanity_death_system);
    }
}

#[derive(Resource, Default)]
pub struct SanityTimers {
    pub timers: Vec<(Entity, Timer, f32)>,
}

pub fn sanity_drain_system(
    time: Res<Time>,
    mut sanity_timers: ResMut<SanityTimers>,
    mut players: Query<(Entity, &NetControl, &Player, &mut Sanity)>,
) {
    const SANITY_DRAIN_RATE: f32 = 5.0; // Drain 5 sanity per second without flashlight
    const SANITY_REGEN_RATE: f32 = 10.0; // Regenerate 10 sanity per second with flashlight
    const MAX_SANITY: f32 = 100.0;

    for (entity, control, player, mut sanity) in players.iter_mut() {
        if control.get_type() != PlayerType::Local {
            continue;
        }

        let delta = time.delta_secs();

        if player.charge > 0 {
            sanity.current = (sanity.current + SANITY_REGEN_RATE * delta).min(MAX_SANITY);
            sanity.draining = false;
            
            sanity_timers.timers.retain(|(timer_entity, _, _)| *timer_entity != entity);
        } else {
            sanity.current = (sanity.current - SANITY_DRAIN_RATE * delta).max(0.0);
            sanity.draining = true;
        }
    }
}


pub fn sanity_death_system(
    mut writer: EventWriter<DamagePlayerEvent>,
    mut players: Query<(Entity, &NetControl, &mut Sanity)>,
) {
    for (entity, control, mut sanity) in players.iter_mut() {
        if control.get_type() != PlayerType::Local {
            continue;
        }
        if sanity.current <= 0.0 && sanity.draining {
            writer.send(DamagePlayerEvent::new(entity, 100));
            sanity.draining = false;
        }
    }
}
