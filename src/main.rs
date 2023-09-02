use cpu::Cpu;
use minifb::{Window, WindowOptions};
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

    let mut f = File::open("roms/bc_test.ch8")?;
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
        let mut should_update = false;
        for _ in 0..700 / 60 {
            should_update |= cpu.tick(&mut memory, &mut stack, &mut pixels);
        }
        if should_update {
            window.update_with_buffer(&pixels, WIDTH, HEIGHT)?;
        } else {
            window.update();
        }
    }

    Ok(())
}
