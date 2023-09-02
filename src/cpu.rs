use crate::{HEIGHT, WIDTH};

#[derive(Default)]
pub(crate) struct Cpu {
    /// A program counter, often called just “PC”, which points at the current instruction in memory
    pc: u16,
    /// One 16-bit index register called “I” which is used to point at locations in memory
    i: u16,
    /// 16 8-bit general-purpose variable registers numbered 0 through F hexadecimal (V0..VF)
    v: [u8; 16],
}

impl Cpu {
    pub(crate) fn tick(&mut self, mem: &mut [u8], screen: &mut [u32]) -> bool {
        let i = u16::from_be_bytes([mem[self.pc as usize], mem[(self.pc as usize) + 1]]);
        self.pc += 2;

        match i & 0xF000 {
            0x0000 => {
                screen.fill(0);
            }
            0x1000 => {
                self.pc = i & 0xFFF;
            }
            0x6000 => {
                let x = (i >> 8) & 0xF;
                self.v[x as usize] = (i & 0xFF) as u8;
            }
            0x7000 => {
                let x = (i >> 8) & 0xF;
                self.v[x as usize] = self.v[x as usize].wrapping_add((i & 0xFF) as u8);
            }
            0xA000 => {
                self.i = i & 0xFFF;
            }
            0xD000 => {
                let x = (i >> 8) & 0xF;
                let y = (i >> 4) & 0xF;
                let vx = self.v[x as usize] as usize % WIDTH;
                let vy = self.v[y as usize] as usize % HEIGHT;

                self.v[0xF] = 0;

                for row in 0..(i & 0xF) as usize {
                    if (vy + row) >= HEIGHT {
                        break;
                    }
                    let byte = mem[self.i as usize + row];
                    for col in 0..8 {
                        if (vx + col) >= WIDTH {
                            break;
                        }
                        let bit = byte & (0x80 >> col);
                        if (bit) != 0 {
                            let index = (vy + row) * WIDTH + vx + col;
                            if screen[index] == 0xFFFF_FFFF {
                                self.v[0xF] = 1;
                            }
                            screen[index] ^= 0xFFFF_FFFF;
                        }
                    }
                }

                return true;
            }
            _ => {}
        }

        return false;
    }
}
