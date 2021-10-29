use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Rect, Point};
use std::time::{Duration, SystemTime};
use std::thread::sleep;
use rand::prelude::*;
use rayon::prelude::*;

const GRID_WIDTH: usize = 2000 ;
const GRID_HEIGHT: usize = 1000;
const GRID_LENGTH: usize = GRID_WIDTH * GRID_HEIGHT;
const CELL_SIZE: u32 = 1;
const BG_COLOUR: Color = Color::RGB(0, 0, 0);
const WINDOW_WIDTH: u32 = (GRID_WIDTH * CELL_SIZE as usize) as u32;
const WINDOW_HEIGHT: u32 = (GRID_HEIGHT * CELL_SIZE as usize) as u32;
const LIVE_VALUE: u8 = 250;
const DEATH_STEP: u8 = 2;
const FPS_TARGET: u32 = 60;

fn val_to_colour(iu: &u8) -> Color {
    let i = *iu as f32 / (LIVE_VALUE as f32);
    let r = (255. * (i * i)).round() as u8;
    let g = (50. * i).round() as u8;
    let b = (255. * i).round() as u8;
    return Color::RGB(r, g, b);
}

fn random_cell_state() -> u8 {
    return if random::<bool>() { LIVE_VALUE } else { 0 };
}

fn coords_to_index(x: i32, y: i32) -> usize {
    return x as usize + (GRID_WIDTH * y as usize);
}

fn index_to_coords_with_grid_width(i: usize, grid_width: usize) -> (i32, i32) {
    let x = (i % grid_width) as i32;
    let y = (i / grid_width) as i32;
    return (x, y);
}

fn index_to_coords(i: usize) -> (i32, i32) {
    return index_to_coords_with_grid_width(i, GRID_WIDTH);
}

fn is_alive(grid: &[u8], x: i32, y: i32) -> bool {
    return 
        x >= 0 && 
        y >= 0 && 
        x < (GRID_WIDTH as i32) &&
        y < (GRID_HEIGHT as i32) &&
        grid[coords_to_index(x, y)] >= LIVE_VALUE;
}

fn count_neighbours(i: usize, grid: &[u8]) -> usize {
    let (x, y) = index_to_coords(i);

    let mut count = 0;
    count += is_alive(grid, x - 1, y - 1) as usize;
    count += is_alive(grid, x, y - 1) as usize;
    count += is_alive(grid, x + 1, y - 1) as usize;
    count += is_alive(grid, x - 1, y) as usize;
    count += is_alive(grid, x + 1, y) as usize;
    count += is_alive(grid, x - 1, y + 1) as usize;
    count += is_alive(grid, x, y + 1) as usize;    
    count += is_alive(grid, x + 1, y + 1) as usize;
    return count;
}

fn step_grid(in_grid: &[u8]) -> [u8; GRID_LENGTH] {
    let mut out_grid = [0; GRID_LENGTH];
    out_grid.par_iter_mut().enumerate().for_each(|(index, value)| {
        let is_alive = in_grid[index] >= LIVE_VALUE;
        let neighbours = count_neighbours(index, in_grid);
        if (is_alive && neighbours == 2) || neighbours == 3 {
            *value = LIVE_VALUE;
        } else if in_grid[index] > 0 {
            *value = in_grid[index] - DEATH_STEP;
        }
    });

    for x in 0..GRID_WIDTH {
        out_grid[coords_to_index(x as i32, 0)] = random_cell_state();
        out_grid[coords_to_index(x as i32, (GRID_HEIGHT - 1) as i32)] = random_cell_state();
    }
    for y in 0..GRID_HEIGHT {
        out_grid[coords_to_index(0, y as i32)] = random_cell_state();
        out_grid[coords_to_index((GRID_WIDTH - 1) as i32, y as i32)] = random_cell_state();
    }

    return out_grid;
}

fn main() -> Result<(), String> {
    assert!(LIVE_VALUE % DEATH_STEP == 0, "DEATH_STEP must fit evenly into LIVE_VALUE");

    let sdl_context = sdl2::init()?;
    let video_subsystem = sdl_context.video()?;
    let window = video_subsystem.window("Game of Life", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .expect("could not initialize video subsystem");

    let mut canvas = window.into_canvas().build()
        .expect("could not make a canvas");
    canvas.set_blend_mode(sdl2::render::BlendMode::None);
    canvas.set_scale(CELL_SIZE as f32, CELL_SIZE as f32).expect("Error scaling canvas");

    canvas.set_draw_color(BG_COLOUR);
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;
    let mut grid = [0; GRID_LENGTH];
    for i in 0..GRID_LENGTH {
        grid[i] = random_cell_state();
    }
    let mut avg_frametime = 0.;
    let mut frame_count = 0.;
    'running: loop {
        let frame_start = SystemTime::now();
        frame_count += 1.;
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    for i in 0..GRID_LENGTH {
                        grid[i] = random_cell_state();
                    }
                },
                _ => {}
            }
        }

        sleep(Duration::new(0, 1_000_000_000 / FPS_TARGET));

        canvas.set_draw_color(BG_COLOUR);
        canvas.clear();
        grid.iter().enumerate().for_each(|(index, &i)| {
            if i < LIVE_VALUE { return; }
            // let draw_start = SystemTime::now();
            let (x, y) = index_to_coords(index);
            canvas.set_draw_color(val_to_colour(&i));
            canvas.draw_point(Point::new(x as i32, y as i32)).unwrap();
            // let draw_time_micros = SystemTime::now().duration_since(draw_start).unwrap().as_micros();
            // if draw_time_micros > 0 {
            //     println!("Draw took {} microseconds", draw_time_micros);
            // }
        });
        canvas.present();

        grid = step_grid(&grid);
        let frame_time = SystemTime::now().duration_since(frame_start).unwrap().as_nanos();
        avg_frametime = ((frame_count * avg_frametime) + frame_time as f64) / (frame_count + 1.);
        if frame_count > 500. {
            println!("Avg frametime: {}ns", avg_frametime);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::{index_to_coords_with_grid_width};

    #[test]
    fn index_to_coords_works() {
        let (x, y) = index_to_coords_with_grid_width(9, 4);
        assert_eq!(x, 1);
        assert_eq!(y, 2);
    }
}