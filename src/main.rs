use cpu::Cpu;
use minifb::{Key, Window, WindowOptions};
use rodio::Sink;
use std::fs::File;
use std::io::Read;

mod cpu;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

const FONT: &[u8] = &[
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

fn main() -> anyhow::Result<()> {
    let mut pixels = [0_u32; WIDTH * HEIGHT];
    let mut memory = [0_u8; 4096];
    let mut cpu = Cpu::default();
    let mut stack: Vec<u16> = Vec::new();
    let mut delay: u8 = 0;
    let mut sound: u8 = 0;

    let (_stream, stream_handle) = rodio::OutputStream::try_default()?;
    let wave = rodio::source::SineWave::new(440.0);
    let sink = Sink::try_new(&stream_handle)?;
    sink.pause();
    sink.append(wave);

    let mut f = File::open("roms/Space Invaders [David Winter].ch8")?;
    f.read(&mut memory[512..])?;

    memory[0x50..=0x9F].copy_from_slice(FONT);

    let mut window = Window::new(
        "CHIP-8",
        WIDTH,
        HEIGHT,
        WindowOptions {
            scale: minifb::Scale::X16,
            ..Default::default()
        },
    )?;

    // Limit to max ~60 fps update rate
    window.limit_update_rate(Some(std::time::Duration::from_micros(1_000_000 / 60)));

    while window.is_open() {
        delay = delay.saturating_sub(1);
        sound = sound.saturating_sub(1);

        let mut keys_pressed = [false; 16];
        keys_pressed[0x1] = window.is_key_down(Key::Key1);
        keys_pressed[0x2] = window.is_key_down(Key::Key2);
        keys_pressed[0x3] = window.is_key_down(Key::Key3);
        keys_pressed[0xC] = window.is_key_down(Key::Key4);

        keys_pressed[0x4] = window.is_key_down(Key::Q);
        keys_pressed[0x5] = window.is_key_down(Key::W);
        keys_pressed[0x6] = window.is_key_down(Key::E);
        keys_pressed[0xD] = window.is_key_down(Key::R);

        keys_pressed[0x7] = window.is_key_down(Key::A);
        keys_pressed[0x8] = window.is_key_down(Key::S);
        keys_pressed[0x9] = window.is_key_down(Key::D);
        keys_pressed[0xE] = window.is_key_down(Key::F);

        keys_pressed[0xA] = window.is_key_down(Key::Z);
        keys_pressed[0x0] = window.is_key_down(Key::X);
        keys_pressed[0xB] = window.is_key_down(Key::C);
        keys_pressed[0xF] = window.is_key_down(Key::V);

        let mut keys_released = [false; 16];
        keys_released[0x1] = window.is_key_released(Key::Key1);
        keys_released[0x2] = window.is_key_released(Key::Key2);
        keys_released[0x3] = window.is_key_released(Key::Key3);
        keys_released[0xC] = window.is_key_released(Key::Key4);

        keys_released[0x4] = window.is_key_released(Key::Q);
        keys_released[0x5] = window.is_key_released(Key::W);
        keys_released[0x6] = window.is_key_released(Key::E);
        keys_released[0xD] = window.is_key_released(Key::R);

        keys_released[0x7] = window.is_key_released(Key::A);
        keys_released[0x8] = window.is_key_released(Key::S);
        keys_released[0x9] = window.is_key_released(Key::D);
        keys_released[0xE] = window.is_key_released(Key::F);

        keys_released[0xA] = window.is_key_released(Key::Z);
        keys_released[0x0] = window.is_key_released(Key::X);
        keys_released[0xB] = window.is_key_released(Key::C);
        keys_released[0xF] = window.is_key_released(Key::V);

        if sound > 0 && sink.is_paused() {
            sink.play();
        } else if sound == 0 && !sink.is_paused() {
            sink.pause();
        }

        let mut should_update = false;
        for _ in 0..700 / 60 {
            should_update |= cpu.tick(
                &mut memory,
                &mut stack,
                &mut pixels,
                &mut delay,
                &mut sound,
                &keys_pressed,
                &keys_released,
            );
        }
        if should_update {
            window.update_with_buffer(&pixels, WIDTH, HEIGHT)?;
        } else {
            window.update();
        }
    }

    Ok(())
}
