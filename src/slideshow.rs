use std::time::Duration;

use bevy::{prelude::*, window::PresentMode};

#[derive(Resource)]
struct MyTimer(Timer);

// This resource will hold all of our images! All 8 of them.
#[derive(Resource)]
struct MyImageArray {
    image_array: [Entity; 8],
}

#[derive(Resource, Default)]
struct MyCounter {
    value: usize,
}

// This is a trait for MyImageArray. It's needed for .init_resource in App::new(), so rust knows what to initialize the array to.
// see more about "Entity::PLACEHOLDER" here: (https://docs.rs/bevy/latest/bevy/ecs/entity/struct.Entity.html)
// (also just a great place for more bevy info)
impl Default for MyImageArray {
    fn default() -> Self {
        MyImageArray {
            image_array: [Entity::PLACEHOLDER; 8],
        }
    }
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Team Blueberry Credits Slideshow".into(),
                resolution: (1280., 720.).into(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            }),
            ..default()
        }))
        .insert_resource(MyTimer(Timer::new(
            Duration::from_secs(2),
            TimerMode::Repeating,
        )))
        .init_resource::<MyImageArray>() // Initializes this array with all placeholder entities! See above for more info...
        .init_resource::<MyCounter>() // Does the same with the counter, initializing to 0.
        .add_systems(Startup, setup)
        .add_systems(Update, countdown)
        .run();
}

// The arguments of this system (that's what Bevy calls functions) takes both an immutable and a mutable resource.
// This is how systems get global information. Global variables must be packaged within resource.
fn setup(mut commands: Commands, asset_server: Res<AssetServer>, mut image_array_resource: ResMut<MyImageArray>) {
    // Spawn the camera...so we can see stuff
    commands.spawn(Camera2d);
    // Time to populate our array!
    // Here we spawn the image entities and assign them to the array. Notice how we're using .id() to get reference to entity.
    // I spawn them underground (at y = -1000.) initially, and bring them to (0., 0., 0.) in countdown()!!
    image_array_resource.image_array[0] = commands.spawn((
        Sprite::from_image(asset_server.load("amyia.png")),
        Transform {
            translation: Vec3::new(0., -1000., 0.),
            ..default()
        },
    )).id();
    image_array_resource.image_array[1] = commands.spawn((
        Sprite::from_image(asset_server.load("daniel.png")),
        Transform {
            translation: Vec3::new(0., -1000., 0.),
            ..default()
        },
    )).id();
    image_array_resource.image_array[2] = commands.spawn((
        Sprite::from_image(asset_server.load("gordon.png")),
        Transform {
            translation: Vec3::new(0., -1000., 0.),
            ..default()
        },
    )).id();
    image_array_resource.image_array[3] = commands.spawn((
        Sprite::from_image(asset_server.load("matthew.png")),
        Transform {
            translation: Vec3::new(0., -1000., 0.),
            ..default()
        },
    )).id();
    image_array_resource.image_array[4] = commands.spawn((
        Sprite::from_image(asset_server.load("ifemi.png")),
        Transform {
            translation: Vec3::new(0., -1000., 0.),
            ..default()
        },
    )).id();
    image_array_resource.image_array[5] = commands.spawn((
        Sprite::from_image(asset_server.load("peter.png")),
        Transform {
            translation: Vec3::new(0., -1000., 0.),
            ..default()
        },
    )).id();
    image_array_resource.image_array[6] = commands.spawn((
        Sprite::from_image(asset_server.load("vlad.png")),
        Transform {
            translation: Vec3::new(0., -1000., 0.),
            ..default()
        },
    )).id();
    image_array_resource.image_array[7] = commands.spawn((
        Sprite::from_image(asset_server.load("secret.png")),
        Transform {
            translation: Vec3::new(0., -1000., 0.),
            ..default()
        },
    )).id();
}

// This system needs a query to a mutable transform (because we want to change the image's position to 0, 0, 0! it spawns underground initially.)
fn countdown(
            mut commands: Commands,
            mut timer: ResMut<MyTimer>,
            time: Res<Time>, 
            mut counter: ResMut<MyCounter>, 
            image_array_resource: ResMut<MyImageArray>, 
            mut query: Query<&mut Transform>) 
{
    // Let's stop when we've got through all 8 images.
    if counter.value >= 8 {
        return;
    }

    // query.get_mut() TRIES to find transform component of current_image.
    // it TRIES...the entity may no longer exist, or it may not have a transform component.
    // so, the Result comes from query.get_mut().
    // As such, we evaluate the Result. If it's "Ok" we get out mutable transform component.
    // The "if let" syntax is used for brevity in stead of a match statement. Read more here!: (https://doc.rust-lang.org/rust-by-example/flow_control/if_let.html)
    let current_image = image_array_resource.image_array[counter.value];
    if let Ok(mut transform) = query.get_mut(current_image) {
        transform.translation.y = 0.;
    }
    println!("Started showing image {}", counter.value);

    timer.0.tick(time.delta());

    if timer.0.finished() {
        commands.entity(current_image).despawn();
        println!("Finished showing image {}", counter.value);
        counter.value += 1;
    }
}