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
            //println!("water_tile.altitude = {}", water_tile.altitude);
            //println!("render_altitutde = {}", render_alt);
            *color = Rgba(render_alt, render_alt, render_alt, 1.0);
            //*color = Rgba(1., 1.0, 1.0, 1.0);
        }


        self.time_since_simulate += time.delta_seconds();
        //println!("time_since_simulate {}", self.time_since_simulate);
        if self.time_since_simulate < DELAY {
            return;
        }
        //println!("simulate");
        self.time_since_simulate = 0.;
       

        let mut total_height = 0.;

        let update_heights = |water_tiles: &WaterTiles, heights: &mut f32, num_parts: &mut u32, x: i32, y: i32, | {
            //println!("Updating heights for {},{}", x, y);
            if y >= 0 && y < MAP_HEIGHT as i32 && x >= 0 && x < MAP_WIDTH as i32 {
                *heights += water_tiles.current[x as usize][y as usize].altitude;
                *num_parts += 1;
                //println!("{},{} within bounds and has a height of {} and num_parts of {}", x, y, *heights, num_parts);
            }
        };

        //println!("Before the pass.");
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                //println!("tile[{}][{}]: {:?}", x, y, water_tiles.current[x][y]);
            }
        }

        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                
                //println!("\nacceleration[{}][{}]", x, y);

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
                //println!("heights for [{}][{}] = {}", x, y, heights);
                //println!("num_parts for [{}][{}] = {}", x, y, num_parts);

                let tile = &mut water_tiles.current[x][y];
                
                // acceleration
                if num_parts != 0 {
                    //println!("heights = {}/{}", heights, num_parts);
                    heights /= num_parts as f32;
                    //println!("heights = {}", heights);

                    if POWER != 1. {
                        //println!("POWER != 1: {}", POWER);
                        //vda[index] += Math.Sign(heights - vd[index]) * (float)Math.Pow(Math.Abs(vd[index] - heights), power) / mass;
                        tile.acceleration += (heights - tile.acceleration).signum() * ((tile.altitude - heights).abs()).powf(POWER) / MASS;
                    }
                    else {
                        //println!("POWER = 1");
                        //vda[index] += -(vd[index] - heights) / mass;
                        tile.acceleration += -(tile.altitude - heights) / MASS;
                    }
                    //println!("tile.acceleration = {}", tile.acceleration);
                }

                // damping
                tile.acceleration -= tile.velocity / tile.sustainability;
                //println!("damping: tile.acceleration = {}", tile.acceleration);

                // limit
                if tile.acceleration > LIMIT {
                    tile.acceleration = LIMIT;
                } else if tile.acceleration < -LIMIT {
                    tile.acceleration = -LIMIT;
                }
                
                //println!("limit: tile.acceleration = {}", tile.acceleration);
            }
        }
        // done calculating force

        //println!("After acceleration pass.");
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                //println!("tile[{}][{}]: {:?}", x, y, water_tiles.current[x][y]);
            }
        }

        let shifting = -total_height / (MAP_WIDTH * MAP_HEIGHT) as f32;
        //println!("Shifting: {}", shifting);

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
        
        //println!("After altitutde pass.");
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                //println!("tile[{}][{}]: {:?}", x, y, water_tiles.current[x][y]);
            }
        }

        // let tile = &mut water_tiles.current[0][0];
        // tile.altitude = 100.;

    }
}




                // // UP
                // x_local = x;
                // y_local = y + 1;
                // if y < MAP_HEIGHT {
                //     heights += water_tiles.current[x][y].altitude;
                //     num_parts += 1;
                // }
                // // UP RIGHT
                // if y > 0 && y < MAP_HEIGHT && x > 0 && x < MAP_WIDTH {
                //     heights += water_tiles.current[x][y].altitude;
                //     num_parts += 1;
                // }
                // // RIGHT
                // if x < MAP_WIDTH - 1 {
                //     heights += water_tiles.current[x+1][y].altitude;
                //     num_parts += 1;
                // }
                // // DOWN RIGHT
                // if y > 0 && x < MAP_WIDTH - 1 {
                //     heights += water_tiles.current[x+1][y-1].altitude;
                //     num_parts += 1;
                // }
                // // DOWN
                // if y > 0 {
                //     heights += water_tiles.current[x][y-1].altitude;
                //     num_parts += 1;
                // }
                // // DOWN LEFT
                // if y > 0 && x > 0 {
                //     heights += water_tiles.current[x-1][y-1].altitude;
                //     num_parts += 1;
                // }
                // // LEFT
                // if x > 0 {
                //     heights += water_tiles.current[x-1][y].altitude;
                //     num_parts += 1;
                // }
                // // UP LEFT
                // if y < MAP_HEIGHT - 1 && x > 0 {
                //     heights += water_tiles.current[x-1][y+1].altitude;
                //     num_parts += 1;
                // }