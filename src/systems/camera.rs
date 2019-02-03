use amethyst::{
    core::timing::Time,
    core::transform::Transform,
    ecs::prelude::{Join, Read, ReadStorage, System, WriteStorage},
    input::InputHandler,
    renderer::{
        Camera,
    },
};

pub struct CameraSystem;

pub const CAMERA_SPEED: f32 = 100.;

impl <'s> System<'s> for CameraSystem {
    type SystemData = (
        ReadStorage<'s, Camera>,
        WriteStorage<'s, Transform>,
        Read<'s, Time>,
        Read<'s, InputHandler<String, String>>,
    );

    fn run(&mut self, (cameras, mut transforms, time, input): Self::SystemData) {
        //println!("updating the camera");

        for (camera, transform) in (&cameras, &mut transforms).join()
        {
            let movement_right = input.axis_value("right");
            let movement_up = input.axis_value("up");

            if let Some(movement) = movement_right {
                transform.translate_x(CAMERA_SPEED * time.delta_seconds() * movement as f32);
            }
            if let Some(movement) = movement_up {
                transform.translate_y(CAMERA_SPEED * time.delta_seconds() * movement as f32);
            }
        }
    }
}