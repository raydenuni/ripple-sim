use amethyst::{
    core::
    {
        timing::Time,        
        shrev::{EventChannel, ReaderId},
    },
    ecs::prelude::{Read, Write, Join, System, SystemData, Resources, ReadStorage, WriteStorage},
    renderer::{Rgba},
    input::{ InputEvent, }
};

use crate::water_game::Water;
use crate::water_game::WaterTiles;
use crate::water_game::MAP_HEIGHT;
use crate::water_game::MAP_WIDTH;

pub const MASS: f32 = 0.1; // Mass of each particle. It is the same for all particles.
pub const LIMIT: f32 = 500.; // Maximum absolute height a particle can reach.
pub const ACTION_RESOLUTION: f32 = 20.; // Resolution of movement of particles.
pub const SUSTAIN: f32 = 1000.; // Anti-damping. Propagation range increases by increasing this variable. Minimum is 1f.
pub const DELAY: f32 = 0.001; // Time-out in seconds for force calculations.
pub const PHASE_1: f32 = 0.; // Current phase value of oscillator1.
pub const PHASE_2: f32 = 0.; // Current phase value of oscillator2.
pub const FREQ_1: f32 = 0.2; // Phase changing rate of oscillator1 per calculation. Frequency increases by increasing this variable.
pub const FREQ2: f32 = 0.2; // Phase changing rate of oscillator2 per calculation. Frequency increases by increasing this variable.
pub const POWER: f32 = 1.0; // Power of the output force exerted on each particle. Natural value is 1.0f

pub struct WaterSimulationSystem
{
    event_reader: Option<ReaderId<InputEvent<String>>>,
    time_since_simulate: f32,
    on: bool,
}

impl WaterSimulationSystem {
    pub fn new() -> Self {
        WaterSimulationSystem { 
            event_reader: None,
            time_since_simulate: 0.,
            on: false,
        }
    }
}

impl<'s> System<'s> for WaterSimulationSystem {
    type SystemData = (
        Write<'s, WaterTiles>,
        ReadStorage<'s, Water>,
        WriteStorage<'s, Rgba>,
        Read<'s, Time>,
        Read<'s, EventChannel<InputEvent<String>>>,
    );

    fn setup(&mut self, res: &mut Resources) {
        Self::SystemData::setup(res);

        self.event_reader = Some(
            res.fetch_mut::<EventChannel<InputEvent<String>>>()
                .register_reader(),
        );
    }

    fn run(&mut self, (mut water_tiles, waters, mut colors, time, events): Self::SystemData) {
        let mut do_sim = false;

        for event in events.read(&mut self.event_reader.as_mut().unwrap()) {
            match event {
                InputEvent::ActionPressed(action) => match action.as_ref() {
                    "tick" => {
                        do_sim = true;
                        self.on = true;
                    }
                    _ => (),
                },
                _ => (),
            }
        }

        // if !do_sim {
        //     return;
        // } 
        
        if !self.on {
            return;
        }

        for (water, color) in (&waters, &mut colors).join() {
            let water_tile = water_tiles.current[water.pos_x][water.pos_y];
            let render_alt = (water_tile.altitude + LIMIT) / (2. * LIMIT);
            *color = Rgba(render_alt, render_alt, render_alt, 1.0);
            //*color = Rgba(1., 1.0, 1.0, 1.0);
        }


        self.time_since_simulate += time.delta_seconds();
        if self.time_since_simulate < DELAY {
            return;
        }
        self.time_since_simulate = 0.;
       

        let mut total_height = 0.;

        let update_heights = |water_tiles: &WaterTiles, heights: &mut f32, num_parts: &mut u32, x: i32, y: i32, | {
            if y >= 0 && y < MAP_HEIGHT as i32 && x >= 0 && x < MAP_WIDTH as i32 {
                *heights += water_tiles.current[x as usize][y as usize].altitude;
                *num_parts += 1;
            }
        };

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                water_tiles.current[x][y].acceleration = 0.;
                total_height += water_tiles.current[x][y].altitude;

                let mut heights = 0.;   // total height of all 8 neighbors
                let mut num_parts = 0;  // number of neighbors that contributed

                for x_local in -1..=1 as i32 {
                    for y_local in -1..=1 as i32 {
                        if x_local != 0 || y_local != 0
                        {
                            update_heights(&water_tiles, &mut heights, &mut num_parts, x as i32 + x_local, y as i32 + y_local);
                        }
                    }
                }

                let tile = &mut water_tiles.current[x][y];
                
                // acceleration
                if num_parts != 0 {
                    heights /= num_parts as f32;

                    if POWER != 1. {
                        //vda[index] += Math.Sign(heights - vd[index]) * (float)Math.Pow(Math.Abs(vd[index] - heights), power) / mass;
                        tile.acceleration += (heights - tile.acceleration).signum() * ((tile.altitude - heights).abs()).powf(POWER) / MASS;
                    }
                    else {
                        //vda[index] += -(vd[index] - heights) / mass;
                        tile.acceleration += -(tile.altitude - heights) / MASS;
                    }
                }

                // damping
                tile.acceleration -= tile.velocity / tile.sustainability;

                // limit
                if tile.acceleration > LIMIT {
                    tile.acceleration = LIMIT;
                } else if tile.acceleration < -LIMIT {
                    tile.acceleration = -LIMIT;
                }
            }
        }
        // done calculating force

        let shifting = -total_height / (MAP_WIDTH * MAP_HEIGHT) as f32;
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                let tile = &mut water_tiles.current[x][y];
                tile.velocity += tile.acceleration;

                if tile.altitude + tile.velocity / ACTION_RESOLUTION > LIMIT {
                    tile.altitude = LIMIT;
                } 
                else if tile.altitude + tile.velocity / ACTION_RESOLUTION <= LIMIT && 
                        tile.altitude + tile.velocity / ACTION_RESOLUTION >= -LIMIT {
                    tile.altitude += tile.velocity / ACTION_RESOLUTION;
                }
                else {
                    tile.altitude = -LIMIT;
                }

                // final shift toward origin
                tile.altitude += shifting;
            }
        }
    }
}