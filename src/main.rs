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
use sdl2::render::Texture;
use sdl2::render::TextureQuery;
use sdl2::mixer::{InitFlag, AUDIO_S16LSB, DEFAULT_CHANNELS};
use std::process::Command;

#[derive(Clone)]
struct Cell {
    x: isize,
    y: isize,
    color: Color,
    age: u64, // Add age to track how long the cell has lived
}

impl Cell {
    fn new(x: isize, y: isize) -> Self {
        let mut rng = rand::thread_rng();
        Self {
            x,
            y,
            color: Color::RGB(rng.r#gen(), rng.r#gen(), rng.r#gen()),
            age: 0, // Initialize age to 0
        }
    }

    fn update(&self, neighbors: usize) -> Option<Self> {
        if neighbors == 2 || neighbors == 3 {
            Some(Self { x: self.x, y: self.y, color: self.color, age: self.age + 1 }) // Increment age
        } else {
            None
        }
    }
}

struct GameOfLife {
    cells: Vec<Cell>,
    generation: u64,
    offset_x: isize,
    offset_y: isize,
    // Remove score field
}

impl GameOfLife {
    fn new() -> Self {
        let mut instance = Self {
            cells: Vec::new(),
            generation: 0,
            offset_x: 0,
            offset_y: 0,
            // Remove score initialization
        };
        let mut rng = rand::thread_rng();
        while instance.cells.len() < 500 {
            let x = rng.gen_range(0..50) as isize;
            let y = rng.gen_range(0..50) as isize;
            if !instance.cells.iter().any(|cell| cell.x == x && cell.y == y) {
                instance.cells.push(Cell::new(x, y));
            }
        }
        instance
    }

    fn update(&mut self) {
        let mut next_cells = Vec::new();
        let mut neighbor_counts = std::collections::HashMap::new();

        for cell in &self.cells {
            for dy in [-1, 0, 1] {
                for dx in [-1, 0, 1] {
                    if dx == 0 && dy == 0 { continue; }
                    let neighbor_x = cell.x + dx;
                    let neighbor_y = cell.y + dy;
                    *neighbor_counts.entry((neighbor_x, neighbor_y)).or_insert(0) += 1;
                }
            }
        }

        for cell in &self.cells {
            let count = neighbor_counts.get(&(cell.x, cell.y)).cloned().unwrap_or(0);
            if let Some(updated_cell) = cell.update(count) {
                // Remove score increment
                next_cells.push(updated_cell);
            }
        }

        for (&(x, y), &count) in &neighbor_counts {
            if count == 3 && !self.cells.iter().any(|cell| cell.x == x && cell.y == y) {
                next_cells.push(Cell::new(x, y));
            }
        }

        self.cells = next_cells;
    }

    fn draw(&self, canvas: &mut sdl2::render::Canvas<sdl2::video::Window>) {
        let (win_w, win_h) = canvas.output_size().unwrap();
        let cell_w = win_w / 50;
        let cell_h = win_h / 50;
        for cell in &self.cells {
            canvas.set_draw_color(cell.color);
            let _ = canvas.fill_rect(Rect::new(
                (cell.x * cell_w as isize + self.offset_x) as i32,
                (cell.y * cell_h as isize + self.offset_y) as i32,
                cell_w as u32,
                cell_h as u32,
            ));
        }
    }

    fn live_cell_count(&self) -> usize {
        self.cells.len()
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

    // Start streaming music from YouTube using youtube-dl and mpv
    let mut mpv_process = Command::new("mpv")
        .arg("--no-video")
        .arg("https://www.youtube.com/watch?v=jfKfPfyJRdk")
        .spawn()
        .expect("Failed to start music stream");

    let window = video_subsystem.window("Conway's Game of Life", 800, 600)
        .fullscreen_desktop() // Start in full-screen mode
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().present_vsync().build().unwrap();
    let mut event_pump = sdl_context.event_pump().unwrap();

    let ttf_context = ttf::init().unwrap();
    let font = ttf_context.load_font("/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf", 20).unwrap();
    let texture_creator = canvas.texture_creator();

    let mut game = GameOfLife::new();

    let mut last_update = Instant::now();
    let mut last_frame = Instant::now();
    let mut fps_count = 0;
    let mut fps = 0;
    let mut dragging = false;
    let mut last_mouse_x = 0;
    let mut last_mouse_y = 0;

    'running: loop {
        let frame_start = Instant::now();
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                    break 'running;
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Left, x, y, .. } => {
                    dragging = true;
                    last_mouse_x = x;
                    last_mouse_y = y;
                },
                Event::MouseButtonUp { mouse_btn: MouseButton::Left, .. } => {
                    dragging = false;
                },
                Event::MouseMotion { x, y, .. } => {
                    if dragging {
                        let dx = x - last_mouse_x;
                        let dy = y - last_mouse_y;
                        game.offset_x += dx as isize;
                        game.offset_y += dy as isize;
                        last_mouse_x = x;
                        last_mouse_y = y;
                    }
                },
                Event::MouseButtonDown { mouse_btn: MouseButton::Right, x, y, .. } => {
                    let (win_w, win_h) = canvas.output_size().unwrap();
                    let cell_w = win_w / 50;
                    let cell_h = win_h / 50;
                    let cx = ((x as isize - game.offset_x) / cell_w as isize) as isize;
                    let cy = ((y as isize - game.offset_y) / cell_h as isize) as isize;
                    let mut rng = rand::thread_rng();
                    for _ in 0..50 {
                        let angle = rng.gen_range(0.0..(2.0 * std::f32::consts::PI));
                        let radius = rng.gen_range(0..5) as f32;
                        let dx = (radius * angle.cos()).round() as isize;
                        let dy = (radius * angle.sin()).round() as isize;
                        let nx = cx + dx;
                        let ny = cy + dy;
                        if !game.cells.iter().any(|cell| cell.x == nx && cell.y == ny) {
                            game.cells.push(Cell::new(nx, ny));
                        }
                    }
                },
                Event::KeyDown { keycode: Some(Keycode::X), .. } => {
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
        // Remove score texture creation and rendering
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

    // Ensure mpv process is killed when the game is closed
    mpv_process.kill().expect("Failed to kill mpv process");
}
