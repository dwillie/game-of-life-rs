use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::{Rect};
use std::time::Duration;
use rand::prelude::*;
use rayon::prelude::*;

const GRID_WIDTH: usize = 2000;
const GRID_HEIGHT: usize = 1000;
const GRID_LENGTH: usize = GRID_WIDTH * GRID_HEIGHT;
const CELL_SIZE: u32 = 1;
const BG_COLOUR: Color = Color::RGBA(0, 0, 0, 100);
const WINDOW_WIDTH: u32 = (GRID_WIDTH * CELL_SIZE as usize) as u32;
const WINDOW_HEIGHT: u32 = (GRID_HEIGHT * CELL_SIZE as usize) as u32;

fn val_to_colour(i: &f32) -> Color {
    let r = (255. * (i * i)).round() as u8;
    let g = (50. * i).round() as u8;
    let b = (255. * i).round() as u8;
    return Color::RGB(r, g, b);
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

fn is_alive(grid: &[f32], x: i32, y: i32) -> bool {
    return 
        x >= 0 && 
        y >= 0 && 
        x < (GRID_WIDTH as i32) &&
        y < (GRID_HEIGHT as i32) &&
        grid[coords_to_index(x, y)] >= 1.;
}

fn count_neighbours(i: usize, grid: &[f32]) -> usize {
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

fn step_grid(in_grid: &[f32]) -> [f32; GRID_LENGTH] {
    let mut out_grid = [0.; GRID_LENGTH];
    out_grid.par_iter_mut().enumerate().for_each(|(index, value)| {
        let is_alive = in_grid[index] >= 1.;
        let neighbours = count_neighbours(index, in_grid);
        if (is_alive && neighbours == 2) || neighbours == 3 {
            *value = 1.;
        } else if in_grid[index] > 0. {
            *value = in_grid[index] - 0.01;
        }
    });

    for x in 0..GRID_WIDTH {
        out_grid[coords_to_index(x as i32, 0)] = random::<f32>() + 0.5;
        out_grid[coords_to_index(x as i32, (GRID_HEIGHT - 1) as i32)] = random::<f32>() + 0.5;
    }
    for y in 0..GRID_HEIGHT {
        out_grid[coords_to_index(0, y as i32)] = random::<f32>() + 0.5;
        out_grid[coords_to_index((GRID_WIDTH - 1) as i32, y as i32)] = random::<f32>() + 0.5;   
    }

    return out_grid;
}

fn main() -> Result<(), String> {
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

    canvas.set_draw_color(BG_COLOUR);
    canvas.clear();
    canvas.present();
    let mut event_pump = sdl_context.event_pump()?;
    let mut grid = [0.; GRID_LENGTH];
    for i in 0..GRID_LENGTH {
        grid[i] = random::<f32>() + 0.5
    }
    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::KeyDown { keycode: Some(Keycode::Space), .. } => {
                    for i in 0..GRID_LENGTH {
                        grid[i] = random::<f32>() + 0.5;
                    }
                },
                _ => {}
            }
        }

        // 1 billion nanoseconds in a second, targeting 60fps.
        // ::std::thread::sleep(Duration::new(0, 1_000_000_000 / 60));

        canvas.set_draw_color(BG_COLOUR);
        canvas.clear();

        grid.iter().enumerate().for_each(|(index, &i)| {
            if i <= 1. { return; }
            let (x, y) = index_to_coords(index);
            let cell_colour = val_to_colour(&i);
            canvas.set_draw_color(cell_colour);
            canvas.fill_rect(
                Rect::new(
                    x * CELL_SIZE as i32,
                    y * CELL_SIZE as i32,
                    CELL_SIZE,
                    CELL_SIZE
                )).unwrap();
        });

        canvas.present();
        grid = step_grid(&grid);
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