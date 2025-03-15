use sdl2::pixels::Color;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::rect::Rect;
use std::time::{Duration, Instant};
use sdl2::mouse::MouseButton;
use sdl2::ttf;
use rand::Rng;
use sdl2::render::TextureCreator;
use sdl2::video::WindowContext;
use sdl2::surface::Surface;
use sdl2::render::Texture;
use sdl2::render::TextureQuery;

struct GameOfLife {
    grid: Vec<Vec<Option<Color>>>,
    generation: u64,
}

impl GameOfLife {
    fn new(width: usize, height: usize) -> Self {
        let mut instance = Self {
            grid: vec![vec![None; width]; height],
            generation: 0,
        };
        let mut rng = rand::thread_rng();
        for _ in 0..100 {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);
            instance.grid[y][x] = Some(Color::RGB(rng.r#gen(), rng.r#gen(), rng.r#gen()));
        }
        instance
    }

    fn update(&mut self) {
        let mut rng = rand::thread_rng();
        let mut next_grid = self.grid.clone();
        let height = self.grid.len();
        let width = if height > 0 { self.grid[0].len() } else { 0 };
        for y in 0..height {
            for x in 0..width {
                let is_alive = self.grid[y][x].is_some();
                let mut neighbors = 0;
                for dy in [-1, 0, 1] {
                    for dx in [-1, 0, 1] {
                        if dx == 0 && dy == 0 { continue; }
                        let ny = (y as isize + dy).rem_euclid(height as isize) as usize;
                        let nx = (x as isize + dx).rem_euclid(width as isize) as usize;
                        if self.grid[ny][nx].is_some() {
                            neighbors += 1;
                        }
                    }
                }
                next_grid[y][x] = if is_alive && (neighbors == 2 || neighbors == 3) {
                    self.grid[y][x]
                } else if !is_alive && neighbors == 3 {
                    Some(Color::RGB(rng.r#gen(), rng.r#gen(), rng.r#gen()))
                } else {
                    None
                };
            }
        }
        self.grid = next_grid;
    }

    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let (win_w, win_h) = canvas.output_size().unwrap();
        let height = self.grid.len();
        let width = if height > 0 { self.grid[0].len() } else { 0 };
        if width == 0 || height == 0 { return; }
        let cell_w = win_w / width as u32;
        let cell_h = win_h / height as u32;
        for (y, row) in self.grid.iter().enumerate() {
            for (x, &maybe_color) in row.iter().enumerate() {
                if let Some(color) = maybe_color {
                    canvas.set_draw_color(color);
                    let _ = canvas.fill_rect(Rect::new(
                        x as i32 * cell_w as i32,
                        y as i32 * cell_h as i32,
                        cell_w,
                        cell_h,
                    ));
                }
            }
        }
    }

    fn live_cell_count(&self) -> usize {
        self.grid.iter().flat_map(|row| row.iter()).filter(|c| c.is_some()).count()
    }
}

fn create_texture_from_text<'a>(
    texture_creator: &'a TextureCreator<WindowContext>,
    font: &sdl2::ttf::Font,
    text: &str,
    color: Color,
) -> Texture<'a> {
    let surface = font.render(text).blended(color).unwrap();
    texture_creator.create_texture_from_surface(&surface).unwrap()
}

fn main() {
    let sdl_context = sdl2::init().unwrap();
    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem.window("Conway's Game of Life", 800, 600)
        .resizable()
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let ttf_context = ttf::init().unwrap();
    let font = ttf_context.load_font("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 20).unwrap();
    let texture_creator = canvas.texture_creator();

    let mut game = GameOfLife::new(50, 50);

    let mut last_update = Instant::now();
    let mut last_frame = Instant::now();
    let mut fps_count = 0;
    let mut fps = 0;

    'running: loop {
        let frame_start = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    let (win_w, win_h) = canvas.output_size().unwrap();
                    let height = game.grid.len();
                    let width = if height > 0 { game.grid[0].len() } else { 0 };
                    let mut rng = rand::thread_rng();
                    if width > 0 && height > 0 {
                        let cell_w = win_w / width as u32;
                        let cell_h = win_h / height as u32;
                        let cx = (x as u32 / cell_w) as usize;
                        let cy = (y as u32 / cell_h) as usize;
                        if cy < height && cx < width && game.grid[cy][cx].is_none() {
                            game.grid[cy][cx] = Some(Color::RGB(rng.r#gen(), rng.r#gen(), rng.r#gen()));
                        }
                    }
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Right, x, y, .. } => {
                    let (win_w, win_h) = canvas.output_size().unwrap();
                    let height = game.grid.len();
                    let width = if height > 0 { game.grid[0].len() } else { 0 };
                    let mut rng = rand::thread_rng();
                    if width > 0 && height > 0 {
                        let cell_w = win_w / width as u32;
                        let cell_h = win_h / height as u32;
                        let cx = (x as u32 / cell_w) as usize;
                        let cy = (y as u32 / cell_h) as usize;
                        for _ in 0..50 {
                            let dx = rng.gen_range(-5..=5);
                            let dy = rng.gen_range(-5..=5);
                            let nx = (cx as isize + dx).rem_euclid(width as isize) as usize;
                            let ny = (cy as isize + dy).rem_euclid(height as isize) as usize;
                            if game.grid[ny][nx].is_none() {
                                game.grid[ny][nx] = Some(Color::RGB(rng.r#gen(), rng.r#gen(), rng.r#gen()));
                            }
                        }
                    }
                },
                Event::Window {..} => {
                    // handle window resizing if needed
                },
                _ => {}
            }
        }

        // update grid every 500ms
        if last_update.elapsed() >= Duration::from_millis(500) {
            game.update();
            game.generation += 1;
            last_update = Instant::now();
        }

        canvas.set_draw_color(Color::BLACK);
        canvas.clear();
        game.draw(&mut canvas);

        // draw HUD with fps, live_cell_count, and generation
        let fps_texture = create_texture_from_text(&texture_creator, &font, &format!("FPS: {}", fps), Color::YELLOW);
        let live_cells_texture = create_texture_from_text(&texture_creator, &font, &format!("Live Cells: {}", game.live_cell_count()), Color::YELLOW);
        let generation_texture = create_texture_from_text(&texture_creator, &font, &format!("Generation: {}", game.generation), Color::YELLOW);

        let TextureQuery { width: fps_width, height: fps_height, .. } = fps_texture.query();
        let TextureQuery { width: live_cells_width, height: live_cells_height, .. } = live_cells_texture.query();
        let TextureQuery { width: generation_width, height: generation_height, .. } = generation_texture.query();

        canvas.copy(&fps_texture, None, Some(Rect::new(10, 10, fps_width, fps_height))).unwrap();
        canvas.copy(&live_cells_texture, None, Some(Rect::new(10, 20 + fps_height as i32, live_cells_width, live_cells_height))).unwrap();
        canvas.copy(&generation_texture, None, Some(Rect::new(10, 30 + fps_height as i32 + live_cells_height as i32, generation_width, generation_height))).unwrap();

        canvas.present();

        fps_count += 1;
        if frame_start.duration_since(last_frame).as_secs() >= 1 {
            fps = fps_count;
            fps_count = 0;
            last_frame = Instant::now();
        }

        // Limit to 60 FPS
        let frame_duration = frame_start.elapsed();
        if frame_duration < Duration::from_millis(16) {
            std::thread::sleep(Duration::from_millis(16) - frame_duration);
        }
    }
}
