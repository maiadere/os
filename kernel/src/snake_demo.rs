use heapless::{Vec, format};

use crate::{
    console::draw_text,
    driver::{
        gpio, timer,
        videocore::framebuffer::{Color, Framebuffer},
    },
    log,
};

pub struct SnakeGame {
    body: Vec<(i8, i8), 1530>,
    dir: Dir,
    food: (i8, i8),
    score: u32,
    over: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

impl SnakeGame {
    pub fn new() -> Self {
        Self {
            body: Vec::from_array([(12, 10), (11, 10), (10, 10)]),
            dir: Dir::Right,
            food: (20, 15),
            score: 0,
            over: false,
        }
    }

    pub fn input(&mut self, d: Dir) {
        match (self.dir, d) {
            (Dir::Up, Dir::Down)
            | (Dir::Down, Dir::Up)
            | (Dir::Left, Dir::Right)
            | (Dir::Right, Dir::Left) => {}
            _ => self.dir = d,
        }
    }

    pub fn update(&mut self) {
        if self.over {
            return;
        }

        let mut head = self.body[0];
        match self.dir {
            Dir::Up => head.1 -= 1,
            Dir::Down => head.1 += 1,
            Dir::Left => head.0 -= 1,
            Dir::Right => head.0 += 1,
        }

        if head.0 < 0 || head.0 >= 51 || head.1 < 0 || head.1 >= 30 || self.body.contains(&head) {
            self.over = true;
            return;
        }

        if head == self.food {
            self.body.push(head);
            self.score += 10;

            let i = (self.food.1 as u32 * 51) + self.food.0 as u32;
            let j = i.wrapping_mul(301).wrapping_add(7) % 1530;
            self.food.0 = (j % 51) as i8;
            self.food.1 = (j / 51) as i8;
        }

        self.body.rotate_right(1);
        self.body[0] = head;
    }

    pub fn draw(&self, fb: &Framebuffer) {
        let _ = fb.fill(Color::new(0, 0, 0));

        let score_text = format!(100; "SCORE: {}", self.score).unwrap();
        draw_text(fb, 450, 0, score_text.as_str());

        if self.over {
            draw_text(fb, 450, 280, "GAME OVER");
            draw_text(fb, 410, 300, "PRESS F TO RESTART");
            return;
        }

        self.draw_block(fb, self.food, Color::new(250, 0, 0));

        for p in &self.body {
            self.draw_block(fb, *p, Color::new(0, 250, 0));
        }
    }

    fn draw_block(&self, fb: &Framebuffer, p: (i8, i8), c: Color) {
        for dy in 0..20 {
            for dx in 0..20 {
                let _ = fb.set_pixel(p.0 as u32 * 20 + dx, p.1 as u32 * 20 + dy, c);
            }
        }
    }
}

pub fn run(fb: &Framebuffer) -> ! {
    let mut game = SnakeGame::new();
    let mut i = 0;

    gpio::set_pull_mode(0, gpio::PullMode::PullUp);
    gpio::set_pull_mode(1, gpio::PullMode::PullUp);
    gpio::set_pull_mode(2, gpio::PullMode::PullUp);
    gpio::set_pull_mode(3, gpio::PullMode::PullUp);
    gpio::set_pull_mode(8, gpio::PullMode::PullUp);

    loop {
        if i % 10 == 0 {
            game.update();
            game.draw(fb);
        }

        i += 1;
        timer::delay_ms(10);

        // log!(100;"pin states: 0: {}, 1: {}, 2: {}, 3: {}, 8: {}\n",
        //     gpio::get_pin_state(0),
        //     gpio::get_pin_state(1),
        //     gpio::get_pin_state(2),
        //     gpio::get_pin_state(3),
        //     gpio::get_pin_state(8));

        if !gpio::get_pin_state(0) {
            game.input(Dir::Up);
        }
        if !gpio::get_pin_state(1) {
            game.input(Dir::Left);
        }
        if !gpio::get_pin_state(2) {
            game.input(Dir::Down);
        }
        if !gpio::get_pin_state(3) {
            game.input(Dir::Right);
        }
        if !gpio::get_pin_state(8) && game.over {
            game = SnakeGame::new();
            i = 0;
        }
    }
}
