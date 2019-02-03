extern crate amethyst;

mod water_game;
mod systems;

use amethyst::{
    core::transform::TransformBundle,
    input::InputBundle,
    prelude::*,
    renderer::{
        ColorMask, DisplayConfig, DrawFlat2D, 
        Pipeline, RenderBundle, Stage, ALPHA
    },
    utils::application_root_dir,
    ui::{DrawUi},
};

fn main() -> amethyst::Result<()> {
    amethyst::Logger::from_config(Default::default())
        .level_for("gfx_device_gl", amethyst::LogLevelFilter::Warn)
        .start();


    let binding_path = "resources/bindings_config.ron"; 

    let path = format!(
        "{}/resources/display_config.ron",
        application_root_dir()
    );
    let config = DisplayConfig::load(&path);

    let pipe = Pipeline::build().with_stage(
        Stage::with_backbuffer()
            .clear_target([0.0, 0.0, 0.0, 1.0], 1.0)
            .with_pass(DrawFlat2D::new().with_transparency(ColorMask::all(), ALPHA, None))
            .with_pass(DrawUi::new())
    );
        
    let input_bundle = InputBundle::<String, String>::new()
        .with_bindings_from_file(binding_path)?;

    let game_data = GameDataBuilder::default()
        .with_bundle(RenderBundle::new(pipe, Some(config)).with_sprite_sheet_processor())?
        .with_bundle(TransformBundle::new())?
        .with_bundle(input_bundle)?
        .with(systems::WaterSimulationSystem::new(), "water_sim_system", &[])
        .with(systems::CameraSystem, "camera_system", &[]);

    let mut game = Application::new("./", crate::water_game::WaterGame, game_data)?;

    game.run();

    Ok(())
}
