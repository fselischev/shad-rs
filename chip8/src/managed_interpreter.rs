use crate::{
    data::Word,
    error::Result,
    image::Image,
    interpreter::{Interpreter, SCREEN_HEIGHT, SCREEN_WIDTH},
    platform::{Key, Platform, Point, Sprite},
    KeyEventKind,
};

use core::time::Duration;
use std::u8;

////////////////////////////////////////////////////////////////////////////////

pub struct FrameBuffer([[bool; SCREEN_WIDTH]; SCREEN_HEIGHT]);

impl Default for FrameBuffer {
    fn default() -> Self {
        Self([[false; SCREEN_WIDTH]; SCREEN_HEIGHT])
    }
}

impl FrameBuffer {
    pub fn is_in_bounds(&self, x: u8, y: u8) -> bool {
        (x as usize) < SCREEN_WIDTH && (y as usize) < SCREEN_HEIGHT
    }

    pub fn iter_rows(&self) -> impl Iterator<Item = &[bool; SCREEN_WIDTH]> {
        self.0.iter()
    }

    pub fn iter_rows_mut(&mut self) -> impl Iterator<Item = &mut [bool; SCREEN_WIDTH]> {
        self.0.iter_mut()
    }
}

////////////////////////////////////////////////////////////////////////////////

pub trait RandomNumberGenerator: FnMut() -> Word {}

impl<R: FnMut() -> Word> RandomNumberGenerator for R {}

////////////////////////////////////////////////////////////////////////////////

pub const KEYPAD_SIZE: usize = 16;
pub const KEYPAD_LAST: u8 = KEYPAD_SIZE as u8 - 1;

#[derive(Default)]
struct ManagedPlatform<R: RandomNumberGenerator> {
    rand: R,
    frame_buffer: FrameBuffer,
    delay_timer: Word,
    sound_timer: Word,
    keypad: [KeyEventKind; KEYPAD_SIZE],
    last_key: Option<Key>,
}

impl<R: RandomNumberGenerator> Platform for ManagedPlatform<R> {
    fn draw_sprite(&mut self, pos: Point, sprite: Sprite) -> bool {
        let mut collision = false;
        let pos = Point(pos.0 % SCREEN_WIDTH as u8, pos.1 % SCREEN_HEIGHT as u8);
        for dl in sprite.iter_pixels() {
            let Point(x, y) = pos + dl;

            if !self.frame_buffer.is_in_bounds(x, y) {
                continue;
            }

            let x = x as usize;
            let y = y as usize;
            collision |= self.frame_buffer.0[y][x];
            self.frame_buffer.0[y][x] ^= true;
        }

        collision
    }

    fn clear_screen(&mut self) {
        self.frame_buffer
            .iter_rows_mut()
            .for_each(|r| r.fill(false));
    }

    fn get_delay_timer(&self) -> Word {
        self.delay_timer
    }

    fn set_delay_timer(&mut self, value: Word) {
        self.delay_timer = value;
    }

    fn set_sound_timer(&mut self, value: Word) {
        self.sound_timer = value;
    }

    fn is_key_down(&self, key: Key) -> bool {
        matches!(self.keypad[key.as_usize()], KeyEventKind::Pressed)
    }

    fn consume_key_press(&mut self) -> Option<Key> {
        self.last_key.take()
    }

    fn get_random_word(&mut self) -> Word {
        (self.rand)()
    }
}

impl<R: RandomNumberGenerator> ManagedPlatform<R> {
    fn new(rand: R) -> Self {
        Self {
            rand,
            frame_buffer: Default::default(),
            keypad: [KeyEventKind::default(); KEYPAD_SIZE],
            last_key: None,
            delay_timer: 0,
            sound_timer: 0,
        }
    }
}

////////////////////////////////////////////////////////////////////////////////

pub struct ManagedInterpreter<R: RandomNumberGenerator> {
    inner: Interpreter<ManagedPlatform<R>>,
    operation_duration: Duration,
    delay_tick_duration: Duration,
    sound_tick_duration: Duration,
}

impl<R: RandomNumberGenerator> ManagedInterpreter<R> {
    pub const DEFAULT_OPERATION_DURATION: Duration = Duration::from_millis(2);
    pub const DEFAULT_DELAY_TICK_DURATION: Duration = Duration::from_nanos(16666667);
    pub const DEFAULT_SOUND_TICK_DURATION: Duration = Duration::from_nanos(16666667);

    pub fn new(image: impl Image, rand: R) -> Self {
        Self::new_with_durations(
            image,
            rand,
            Self::DEFAULT_OPERATION_DURATION,
            Self::DEFAULT_DELAY_TICK_DURATION,
            Self::DEFAULT_SOUND_TICK_DURATION,
        )
    }

    pub fn new_with_durations(
        image: impl Image,
        rand: R,
        operation_duration: Duration,
        delay_tick_duration: Duration,
        sound_tick_duration: Duration,
    ) -> Self {
        Self {
            inner: Interpreter::new(image, ManagedPlatform::new(rand)),
            operation_duration,
            delay_tick_duration,
            sound_tick_duration,
        }
    }

    fn decrement_delay_timer(&mut self) {
        if self.inner.platform().delay_timer > 0 {
            self.inner.platform_mut().delay_timer -= 1;
        }
    }

    fn decrement_sound_timer(&mut self) {
        if self.inner.platform().sound_timer > 0 {
            self.inner.platform_mut().sound_timer -= 1;
        }
    }

    pub fn simulate_one_instruction(&mut self) -> Result<()> {
        self.inner.run_next_instruction()
    }

    pub fn simulate_duration(&mut self, mut duration: Duration) -> Result<()> {
        loop {
            let min_dur = self
                .delay_tick_duration
                .min(self.sound_tick_duration.min(self.operation_duration));

            if min_dur > duration {
                self.delay_tick_duration -= duration;
                self.sound_tick_duration -= duration;
                self.operation_duration -= duration;
                break;
            }

            if min_dur == self.delay_tick_duration {
                self.decrement_delay_timer();
                self.delay_tick_duration = Self::DEFAULT_DELAY_TICK_DURATION;
                self.decrement_sound_timer();
                self.sound_tick_duration = Self::DEFAULT_SOUND_TICK_DURATION;
            } else {
                self.delay_tick_duration -= min_dur;
                self.sound_tick_duration -= min_dur;
            }

            if min_dur == self.operation_duration {
                self.simulate_one_instruction()?;
                self.operation_duration = Self::DEFAULT_OPERATION_DURATION;
            } else {
                self.operation_duration -= min_dur;
            }

            duration -= min_dur;
        }
        Ok(())
    }

    pub fn frame_buffer(&self) -> &FrameBuffer {
        &self.inner.platform().frame_buffer
    }

    pub fn set_key_down(&mut self, key: Key, is_down: bool) {
        if is_down {
            let platform = self.inner.platform_mut();
            platform.keypad[key.as_usize()] = KeyEventKind::Pressed;
            platform.last_key = Some(key);
        } else {
            self.inner.platform_mut().keypad[key.as_usize()] = KeyEventKind::Released;
        }
    }
}
