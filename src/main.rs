use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::render::Texture;
use sdl2::surface::Surface;
use num_complex::Complex64;
use std::thread;
use std::time::Instant;
mod colormaps;

const SCREEN_SIZE: (i32, i32) = (1200, 900);

const N_ITERATIONS: i32 = 40;

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("mandelbrot set", SCREEN_SIZE.0 as u32, SCREEN_SIZE.1 as u32)
        .position_centered()
        .build()
        .unwrap();

    let surface = Surface::new(SCREEN_SIZE.0 as u32, SCREEN_SIZE.1 as u32, window.window_pixel_format()).unwrap();

    let mut canvas = window.into_canvas().build().unwrap();

    let texture_creator = canvas.texture_creator();

    let mut texture = Texture::from_surface(&surface, &texture_creator).unwrap();

    let mut scale: f64 = 200.0;
    let mut position: (i32, i32) = (0, 0);

    draw_mandelbrot(&mut texture, position, scale, colormaps::MAGMA_COLORMAP);
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
                        scale *= 1.2;
                        position.0 = (position.0 as f64 * 1.2) as i32;
                        position.1 = (position.1 as f64 * 1.2) as i32;
                    } else {
                        scale /= 1.2;
                        position.0 = (position.0 as f64 / 1.2) as i32;
                        position.1 = (position.1 as f64 / 1.2) as i32;
                    }
                    draw_mandelbrot(&mut texture, position, scale, colormaps::MAGMA_COLORMAP);
                    canvas.present();
                },
                Event::MouseMotion { mousestate, xrel, yrel, .. } => {
                    if mousestate.left() {
                        position.0 += xrel;
                        position.1 += yrel;
                        draw_mandelbrot(&mut texture, position, scale, colormaps::MAGMA_COLORMAP);
                        canvas.present();
                    }
                }
                _ => {}
            }
        }
    }
}

fn z(n: i32, c: Complex64) -> i32 {
    let mut z = Complex64::from(0.0);
    for i in 0..n {
        if z.norm_sqr() > 4.0 {
            return i
        }
        z = (z * z) + c;
    }
    return 2
}

fn draw_mandelbrot(texture: &mut Texture/*canvas: &mut sdl2::render::Canvas<sdl2::video::Window>*/, position: (i32, i32), scale: f64, colormap: [[f32; 3]; 256]) -> () {
    let start_time = Instant::now();
    // canvas.set_draw_color(Color::RGB(0, 0, 0));
    // canvas.clear();
    let n_threads = thread::available_parallelism().unwrap().get() as i32;
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
                                (((x + (i * (SCREEN_SIZE.0 / n_threads))) as f64) / scale) - (((position.0 + (SCREEN_SIZE.0 / 2)) as f64) / scale), 
                                (y as f64 / scale) - (((position.1 + (SCREEN_SIZE.1 / 2)) as f64) / scale)
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
        // let mut prev_c = 0.0;
        // canvas.set_draw_color(color_from_colormap(0.0, colormap));
        let mut data = [0u8; (SCREEN_SIZE.0 * SCREEN_SIZE.1) as usize * 3];
        for (x, column) in thread_result.iter().enumerate() {
            for (y, c) in column.iter().enumerate() {
                let color = bytes_from_colormap(*c, colormap);
                let index = (((x as i32 + ((n_threads - i as i32 - 1) * (SCREEN_SIZE.0 / n_threads))) as usize) + (y * SCREEN_SIZE.0 as usize)) * 3;
                data[index] = color[0];
                data[index + 1] = color[1];
                data[index + 2] = color[2];
                // if *c != prev_c {
                //     canvas.set_draw_color(color_from_colormap(*c, colormap));
                // }
                // canvas.draw_point(((x as i32 + ((n_threads - i as i32 - 1) * (SCREEN_SIZE.0 / n_threads))) as i32, y as i32)).unwrap();
                // prev_c = *c;
            }
        }
        texture.update(None, &data, SCREEN_SIZE.0 as usize).unwrap();
    }
    
    let duration = start_time.elapsed();
    println!("took {} secs to render", duration.as_secs_f64());

}

fn color_from_colormap(factor: f64, colormap: [[f32; 3]; 256]) -> Color {
    let color = colormap[(factor * 255.0) as usize];
    return Color::RGB((color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8)
}

fn bytes_from_colormap(factor: f64, colormap: [[f32; 3]; 256]) -> [u8; 3] {
    let color = colormap[(factor * 255.0) as usize];
    return [(color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8]
}
