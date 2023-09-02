use cpu::Cpu;
use minifb::{Window, WindowOptions};
use std::fs::File;
use std::io::Read;

mod cpu;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

fn main() -> anyhow::Result<()> {
    let mut pixels = [0_u32; WIDTH * HEIGHT];
    let mut memory = [0_u8; 4096];
    let mut cpu = Cpu::default();

    let mut f = File::open("roms/IBM Logo.ch8")?;
    f.read(&mut memory[512..])?;

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
            should_update |= cpu.tick(&mut memory, &mut pixels);
        }
        if should_update {
            window.update_with_buffer(&pixels, WIDTH, HEIGHT)?;
        } else {
            window.update();
        }
    }

    Ok(())
}
