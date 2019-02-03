use amethyst::{
    assets::{AssetStorage, Loader, ProgressCounter},
    core::transform::Transform,
    ecs::prelude::{Component, VecStorage},
    prelude::*,
    renderer::{
        Camera, PngFormat, Projection, Rgba, 
        SpriteRender, SpriteSheet, SpriteSheetFormat,
        SpriteSheetHandle, Texture, TextureMetadata,
    },
};

pub const RENDER_WIDTH: f32 = 720.;
pub const RENDER_HEIGHT: f32 = 720.;
pub const MAP_WIDTH: usize = 128;
pub const MAP_HEIGHT: usize = 128;

pub const TILE_WIDTH: f32 = 4.;

pub struct WaterGame;
impl SimpleState for WaterGame {

    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        
        let tile_sheet_handle = load_sprite_sheet_by_name(world, String::from("texture/tiles_sheet.png"), String::from("texture/tiles_sheet.ron"));

        let mut water_tiles = WaterTiles::default();

        for x in 0..(MAP_WIDTH/2) {
            water_tiles.current[x][0].altitude = 500.;
            water_tiles.current[x][1].altitude = 500.;
            water_tiles.current[x][2].altitude = 500.;
        }

        world.add_resource(water_tiles);

        initialise_camera(world);
        initialise_tilemap(world, tile_sheet_handle.clone());
    }
}

#[derive(Copy, Clone)]
pub struct Water {
    pub pos_x: usize,
    pub pos_y: usize,
}

impl Component for Water {
    type Storage = VecStorage<Self>;
}

#[derive(Copy, Clone, Debug)]
pub struct WaterTile {
    pub altitude: f32,
    pub acceleration: f32,
    pub velocity: f32,
    pub sustainability: f32,
}

impl WaterTile {

}

impl Default for WaterTile {
    fn default() -> Self {
        WaterTile {
            altitude: 0.,
            acceleration: 0.,
            velocity: 0.,
            sustainability: 1000.,
        }
    }
}

#[derive(Clone, Debug)]
pub struct WaterTiles {
    pub current: Vec<Vec<WaterTile>>,
}

impl Default for WaterTiles {
    fn default() -> Self {
        WaterTiles {
            current: vec![vec![WaterTile::default(); MAP_HEIGHT]; MAP_WIDTH],
        }
    }
}

fn load_sprite_sheet_by_name(world: &mut World, sheet_name: String, ron_name: String) -> SpriteSheetHandle {    
    let mut progress = ProgressCounter::new();
    let texture_handle = {
        let loader = world.read_resource::<Loader>();
        let texture_storage = world.read_resource::<AssetStorage<Texture>>();
        loader.load(
            sheet_name,
            PngFormat,
            TextureMetadata::srgb_scale(),
            &mut progress,
            &texture_storage,
        )
    };
    
    let loader = world.read_resource::<Loader>();
    let sprite_sheet_store = world.read_resource::<AssetStorage<SpriteSheet>>();
    loader.load(
            ron_name,
            SpriteSheetFormat,
            texture_handle,
            &mut progress,
            &sprite_sheet_store,
    )
}

fn initialise_camera(world: &mut World) {
    let mut transform = Transform::default();
    transform.set_z(1.0);
    //transform.set_x(-100.);
    //transform.set_y(-100.);

    world
        .create_entity()
        .with(Camera::from(Projection::orthographic(
            0.0,
            RENDER_WIDTH as f32,
            0.0,
            RENDER_HEIGHT as f32,
        )))
        .with(transform)
        .build();
}

fn initialise_tilemap(world: &mut World, sprite_sheet_handle: SpriteSheetHandle) {
    for x in 0..MAP_WIDTH {
        for y in 0..MAP_HEIGHT {
            let sprite_render = SpriteRender {
                sprite_sheet: sprite_sheet_handle.clone(),
                sprite_number: 9,
            };

            let mut local_transform = Transform::default();
            let pos_x = ((x as f32) * TILE_WIDTH + TILE_WIDTH/2.) as f32;
            let pos_y = ((y as f32) * TILE_WIDTH + TILE_WIDTH/2.) as f32;
            let color: Rgba = Rgba(1., 1., 1.0, 1.0);
            local_transform.set_xyz(pos_x, pos_y, 0.);

            world
                .create_entity()
                .with(sprite_render)
                .with(Water { pos_x: x, pos_y: y })
                .with(local_transform)
                .with(color)
                .build();
        }
    }
}