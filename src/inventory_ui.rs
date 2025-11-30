use crate::collectible::PlayerInventory;
use bevy::prelude::*;

#[derive(Component)]
pub struct ReviveKitUI;

#[derive(Component)]
pub struct ReviveKitCounter;

pub fn setup_revive_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    inventory: Res<PlayerInventory>,
) {
    // Root node for the revive kit UI (top-right corner)
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(20.0),
                right: Val::Px(20.0),
                // Let the node auto-size based on children
                flex_direction: FlexDirection::Row, 
                align_items: AlignItems::Center,    
                justify_content: JustifyContent::Start,
                ..default()
            },
            ReviveKitUI,
        ))
        .with_children(|parent| {
            // Icon
            parent.spawn((
                ImageNode::new(asset_server.load("revive kit/Revive Kit_albedo.png")),
                Node {
                    width: Val::Px(200.0),
                    height: Val::Px(200.0),
                    margin: UiRect {
                        right: Val::Px(-50.0), // Move picture right to align with text
                        ..default()
                    },
                    ..default()
                },
            ));

            // Counter text next to icon with some spacing
            parent.spawn((
                Text::new(format!("x{}", inventory.revive_kits)),
                TextFont {
                    font_size: 32.0,
                    ..default()
                },
                TextColor(Color::WHITE),
                ReviveKitCounter,
                Node {
                    margin: UiRect {
                        // Adjust these for alignment
                        top: Val::Px(100.0), 
                        bottom: Val::Px(0.0),
                        ..default()
                    },
                    ..default()
                },
            ));
        })
        .insert(Visibility::Hidden);
}


pub fn update_revive_ui(
    inventory: Res<PlayerInventory>,
    mut query: Query<&mut Visibility, With<ReviveKitUI>>,
    mut counter_q: Query<&mut Text, With<ReviveKitCounter>>,
) {
    // Show or hide UI depending on revive kits
    if let Ok(mut visibility) = query.single_mut() {
        *visibility = if inventory.revive_kits > 0 {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    // Update counter text
    if let Ok(mut text) = counter_q.single_mut() {
        *text = Text::new(format!("x{}", inventory.revive_kits));
    }
}
