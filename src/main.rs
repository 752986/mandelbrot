use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use num_complex::Complex64;
mod colormaps;

const SCREEN_SIZE: (u32, u32) = (1200, 900);

const N_ITERATIONS: i32 = 40;

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
    canvas.set_draw_color(Color::RGB(0, 0, 0));
    canvas.clear();
    for x in 0..canvas.output_size().unwrap().0 {
        for y in 0..canvas.output_size().unwrap().1 {
            canvas.set_draw_color(
                color_from_colormap(
                    z(
                        N_ITERATIONS, 
                        Complex64::new(
                            ((x as i32 - position.0) as f64) / scale, 
                            ((y as i32 - position.1 / 2) as f64) / scale
                        )
                    ) as f64 / N_ITERATIONS as f64,
                    colormap
                )
            );
            canvas.draw_point((x as i32, y as i32)).unwrap();
        }
    }
}

fn color_from_colormap(factor: f64, colormap: [[f32; 3]; 256]) -> Color{
    let color = colormap[(factor * 255.0) as usize];
    return Color::RGB((color[0] * 255.0) as u8, (color[1] * 255.0) as u8, (color[2] * 255.0) as u8)
}

