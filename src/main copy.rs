use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use num_complex::Complex64;
use std::thread;
use std::time::Instant;
mod colormaps;

const SCREEN_SIZE: (u32, u32) = (1200, 900);

const N_ITERATIONS: i32 = 100;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("mandelbrot set", SCREEN_SIZE.0, SCREEN_SIZE.1)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let mut scale: f64 = 200.0;
    let position: (i32, i32) = (SCREEN_SIZE.0 as i32 / 2, SCREEN_SIZE.1 as i32);

    draw_mandelbrot(&mut canvas, position, scale, colormaps::MAGMA_COLORMAP);
    canvas.present();

    let mut event_pump = sdl_context.event_pump().unwrap();
    'main: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. } | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'main
                },
                Event::MouseWheel { y, .. } => {
                    if y > 0 {
                        scale *= 1.5;
                    } else {
                        scale *= 1.0 / 1.5;
                    }
                    draw_mandelbrot(&mut canvas, position, scale, colormaps::MAGMA_COLORMAP);
                    canvas.present();
                }
                _ => {}
            }
        }
    }
}

fn z(n: i32, c: Complex64) -> i32 {
    let mut z = Complex64::from(0.0);
    for i in 0..n {
        if z.norm() > 2.0 {
            return i
        }
        z = (z * z) + c;
    }
    return 0
}

fn draw_mandelbrot(canvas: &mut sdl2::render::Canvas<sdl2::video::Window>, position: (i32, i32), scale: f64, colormap: [[f32; 3]; 256]) -> () {
    let start_time = Instant::now();
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    let n_threads = thread::available_parallelism().unwrap().get() as u32;
    let mut thread_handles: Vec<thread::JoinHandle<Vec<[f64; SCREEN_SIZE.1 as usize]>>> = Vec::with_capacity(n_threads as usize);
    for i in 0..n_threads {
        thread_handles.push(
            thread::spawn(move || {
                let mut output: Vec<[f64; SCREEN_SIZE.1 as usize]> = Vec::with_capacity((SCREEN_SIZE.0 / n_threads) as usize);
                for x in 0..SCREEN_SIZE.0 / n_threads {
                    let mut column = [0.0; SCREEN_SIZE.1 as usize];
                    for y in 0..SCREEN_SIZE.1 {
                        column[y as usize] = z(
                            N_ITERATIONS, 
                            Complex64::new(
                                (((x + (i * (SCREEN_SIZE.0 / n_threads))) as i32 - position.0) as f64) / scale, 
                                ((y as i32 - position.1 / 2) as f64) / scale
                            )
                        ) as f64 / N_ITERATIONS as f64;
                    }
                    output.push(column);
                }
                return output
            })
        );
    }
    for i in 0..thread_handles.len() {
        let t = thread_handles.pop().unwrap();
        let thread_result = t.join().unwrap();
        for (x, column) in thread_result.iter().enumerate() {
            for (y, c) in column.iter().enumerate() {
                canvas.set_draw_color(color_from_colormap(*c, colormap));
                canvas.draw_point(((x + ((n_threads as usize - i - 1) * ((SCREEN_SIZE.0 / n_threads) as usize))) as i32, y as i32)).unwrap();
            }
        }
    }
    
    let duration = start_time.elapsed();
    println!("took {} secs to render", duration.as_secs_f64());

}

fn color_from_colormap(factor: f64, colormap: [[f32; 3]; 256]) -> Color{
    let color = colormap[(factor * 255.0) as usize];
    return Color::RGB((color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8)
}

