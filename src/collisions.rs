use crate::{GameState, components::KinematicCollider, components::StaticCollider};
use bevy::{math::bounding::Aabb2d, math::bounding::IntersectsVolume, prelude::*};

pub struct CollisionsPlugin;
impl Plugin for CollisionsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, do_collisions.run_if(in_state(GameState::Playing)));
    }
}

fn find_mtv(pushee: &Aabb2d, pusher: &Aabb2d) -> Vec2 {
    let a_min = pushee.min;
    let a_max = pushee.max;
    let b_min = pusher.min;
    let b_max = pusher.max;

    // Calculate the overlap on each axis
    let overlap_x1 = b_max.x - a_min.x; // pushing right
    let overlap_x2 = a_max.x - b_min.x; // pushing left
    let overlap_x = if overlap_x1.abs() < overlap_x2.abs() {
        overlap_x1
    } else {
        -overlap_x2
    };

    let overlap_y1 = b_max.y - a_min.y; // pushing up
    let overlap_y2 = a_max.y - b_min.y; // pushing down
    let overlap_y = if overlap_y1.abs() < overlap_y2.abs() {
        overlap_y1
    } else {
        -overlap_y2
    };

    // Choose the axis with the least penetration
    if overlap_x.abs() < overlap_y.abs() {
        return Vec2::new(overlap_x, 0.0);
    } else {
        return Vec2::new(0.0, overlap_y);
    }
}

pub fn do_collisions(
    kinematics: Query<(&KinematicCollider, &mut Transform), Without<StaticCollider>>,
    statics: Query<(&StaticCollider, &Transform), Without<KinematicCollider>>,
) {
    for (kc, mut kt) in kinematics {
        for (sc, st) in &statics {
            let mut transformed_kc_shape = kc.shape.clone();
            transformed_kc_shape.min += kt.translation.truncate();
            transformed_kc_shape.max += kt.translation.truncate();

            let mut transformed_sc_shape = sc.shape.clone();
            transformed_sc_shape.min += st.translation.truncate();
            transformed_sc_shape.max += st.translation.truncate();

            let colliding = transformed_kc_shape.intersects(&transformed_sc_shape);
            if colliding {
                kt.translation = kt.translation
                    + find_mtv(&transformed_kc_shape, &transformed_sc_shape).extend(0.);
            }
        }
    }
}
