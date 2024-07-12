use std::u8;

use raylib::prelude::*;

const WIDTH: usize = 256;
const HEIGHT: usize = 256;
const CELL_SIZE: usize = 4;

#[derive(Clone)]
struct ScreenBuf([u8; (WIDTH * HEIGHT) as usize / 4]);

impl ScreenBuf {
    pub fn new() -> Self {
        ScreenBuf([0; (WIDTH * HEIGHT) as usize / 4])
    }

    #[inline]
    pub fn set(&mut self, x: i32, y: i32, state: u8) {
        let x = (x.rem_euclid(WIDTH as i32)) as usize;
        let y = (y.rem_euclid(HEIGHT as i32)) as usize;

        let index = y * WIDTH + x;
        let byte = index / 4;
        let bit = (index % 4) * 2; // 2 bits per state
        self.0[byte] = (self.0[byte] & !(0b11 << bit)) | ((state & 0b11) << bit);
    }

    #[inline]
    pub fn value_at(&self, x: i32, y: i32) -> u8 {
        let x = (x.rem_euclid(WIDTH as i32)) as usize;
        let y = (y.rem_euclid(HEIGHT as i32)) as usize;

        let index = y * WIDTH + x;
        let byte = index / 4;
        let bit = (index % 4) * 2; // 2 bits per state
        unsafe { *self.0.get_unchecked(byte) >> bit & 0b11 }
    }

    #[inline]
    pub fn neighbours(&self, x: i32, y: i32) -> [u8; 8] {
        [
            self.value_at(x - 1, y - 1),
            self.value_at(x, y - 1),
            self.value_at(x + 1, y - 1),
            self.value_at(x - 1, y),
            self.value_at(x + 1, y),
            self.value_at(x - 1, y + 1),
            self.value_at(x, y + 1),
            self.value_at(x + 1, y + 1),
        ]
    }

    pub fn tick(&self, next_buf: &mut ScreenBuf) {
        for y in 0..HEIGHT as i32 {
            for x in 0..WIDTH as i32 {
                let neighbours = self.neighbours(x, y);
                let count = neighbours.iter().filter(|&&n| n == 1).count();
                let cell = self.value_at(x, y);
                next_buf.set(
                    x,
                    y,
                    match cell {
                        0 => {
                            if count == 2 {
                                1
                            } else {
                                0
                            }
                        }
                        1 => {
                            if count == 3 || count == 4 || count == 5 {
                                1
                            } else {
                                2
                            }
                        }
                        2 => 3,
                        3 => 0,
                        _ => unreachable!(),
                    },
                );
            }
        }
    }

    pub fn to_color_buffer(&self) -> Vec<u8> {
        let mut data = vec![0u8; WIDTH * HEIGHT * CELL_SIZE * CELL_SIZE * 4]; // RGBA format
        for y in 0..HEIGHT {
            for x in 0..WIDTH {
                let color = match self.value_at(x as i32, y as i32) {
                    0 => Color::BLACK,
                    1 => Color::WHITE,
                    2 => Color::GRAY,
                    3 => Color::DARKGRAY,
                    _ => unreachable!(),
                };
                for dy in 0..CELL_SIZE {
                    for dx in 0..CELL_SIZE {
                        let index =
                            ((y * CELL_SIZE + dy) * WIDTH * CELL_SIZE + (x * CELL_SIZE + dx)) * 4;
                        data[index] = color.r;
                        data[index + 1] = color.g;
                        data[index + 2] = color.b;
                        data[index + 3] = color.a;
                    }
                }
            }
        }
        data
    }
}

fn main() {
    let i_width = (WIDTH * CELL_SIZE) as i32;
    let i_height = (HEIGHT * CELL_SIZE) as i32;

    let (mut rl, thread) = raylib::init()
        .size(i_width, i_height)
        .title("starwars ca")
        .build();

    rl.set_target_fps(60);

    let mut current_buf = ScreenBuf::new();
    current_buf.set(WIDTH as i32 / 2, HEIGHT as i32 / 2, 1);
    current_buf.set(WIDTH as i32 / 2 - 1, HEIGHT as i32 / 2, 1);

    let mut next_buf = ScreenBuf::new();

    // Create an image and texture
    let image = Image::gen_image_color(i_width, i_height, Color::BLACK);
    let mut texture = rl.load_texture_from_image(&thread, &image).unwrap();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);

        d.clear_background(Color::BLACK);

        let color_buffer = current_buf.to_color_buffer();
        texture.update_texture(&color_buffer);

        d.draw_texture(&texture, 0, 0, Color::WHITE);

        current_buf.tick(&mut next_buf);
        std::mem::swap(&mut current_buf, &mut next_buf);
    }
}
