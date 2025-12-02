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
    for (entity, control, player, mut sanity) in players.iter_mut() {
        if control.get_type() != PlayerType::Local {
            continue;
        }

        if player.charge <= 0 {
            let mut found = false;

            for timer_entry in sanity_timers.timers.iter_mut() {
                let timer_entity = timer_entry.0;
                let timer = &mut timer_entry.1;
                let start_sanity = timer_entry.2;

                if timer_entity == entity {
                    timer.tick(time.delta());
                    let elapsed = timer.elapsed_secs();
                    if elapsed >= 5.0 {
                        sanity.current = 0.0;
                    } else {
                        sanity.current = start_sanity * (1.0 - elapsed / 5.0);
                    }
                    found = true;
                    break;
                }
            }

            if !found {
                let new_timer = Timer::from_seconds(5.0, TimerMode::Once);
                sanity_timers.timers.push((entity, new_timer, sanity.current));
            }
        } else {
            sanity_timers.timers.retain(|(timer_entity, _, _)| *timer_entity != entity);
        }
    }
}


pub fn sanity_death_system(
    mut writer: EventWriter<DamagePlayerEvent>,
    players: Query<(Entity, &NetControl, &Sanity)>,
) {
    for (entity, control, sanity) in players.iter() {
        if control.get_type() != PlayerType::Local {
            continue;
        }
        if sanity.current <= 0.0 {
            // Just send a ton of damage to execute the player when sanity has depleted
            writer.send(DamagePlayerEvent::new(entity, 9999));
        }
    }
}
