use crate::{HEIGHT, WIDTH};

pub(crate) struct Cpu {
    /// A program counter, often called just “PC”, which points at the current instruction in memory
    pc: u16,
    /// One 16-bit index register called “I” which is used to point at locations in memory
    i: u16,
    /// 16 8-bit general-purpose variable registers numbered 0 through F hexadecimal (V0..VF)
    v: [u8; 16],
    /// State for the Xorshift random number generator
    rand_state: u64,
    /// Whether the CPU is in COSMIC VIP mode (a number of opcodes are slightly different)
    cosmic: bool,
}

impl Cpu {
    pub(crate) fn new(cosmic: bool) -> Self {
        let mut buf = [0; std::mem::size_of::<u64>()];
        getrandom::getrandom(&mut buf).expect("Initial random number generation failed");
        Self {
            pc: 512,
            i: 0,
            v: [0; 16],
            rand_state: u64::from_ne_bytes(buf),
            cosmic,
        }
    }
}

impl Default for Cpu {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Cpu {
    pub(crate) fn tick(
        &mut self,
        memory: &mut [u8],
        stack: &mut Vec<u16>,
        screen: &mut [u32],
        delay: &mut u8,
        sound: &mut u8,
        keys_pressed: &[bool; 16],
        keys_released: &[bool; 16],
    ) -> bool {
        let i = u16::from_be_bytes([memory[self.pc as usize], memory[(self.pc as usize) + 1]]);
        let x = ((i >> 8) & 0xF) as usize;
        let y = ((i >> 4) & 0xF) as usize;
        let nnn = i & 0xFFF;
        let nn = (i & 0xFF) as u8;
        let n = (i & 0xF) as u8;
        self.pc += 2;

        match i & 0xF000 {
            0x0000 => match i {
                0x00E0 => screen.fill(0),
                0x00EE => {
                    self.pc = stack
                        .pop()
                        .expect("ROM attempted to return from empty stack");
                }
                _ => {}
            },
            // 1NNN
            0x1000 => {
                self.pc = nnn;
            }
            // 2NNN
            0x2000 => {
                stack.push(self.pc);
                self.pc = nnn;
            }
            // 3XNN
            0x3000 => {
                if self.v[x] == nn {
                    self.pc += 2;
                }
            }
            // 4XNN
            0x4000 => {
                if self.v[x] != nn {
                    self.pc += 2;
                }
            }
            // 5XY0
            0x5000 => {
                if self.v[x] == self.v[y] {
                    self.pc += 2;
                }
            }
            // 6XNN
            0x6000 => {
                self.v[x] = nn;
            }
            // 7XNN
            0x7000 => {
                self.v[x] = self.v[x].wrapping_add(nn);
            }
            // 8XYN
            0x8000 => match n {
                // Set
                0x0 => self.v[x] = self.v[y],
                // Binary OR
                0x1 => self.v[x] |= self.v[y],
                // Binary AND
                0x2 => self.v[x] &= self.v[y],
                // Logical XOR
                0x3 => self.v[x] ^= self.v[y],
                // Add
                0x4 => {
                    let (res, overflow) = self.v[x].overflowing_add(self.v[y]);
                    self.v[x] = res;
                    self.v[0xF] = overflow as u8;
                }
                // Subtract
                0x5 => {
                    let (res, overflow) = self.v[x].overflowing_sub(self.v[y]);
                    self.v[x] = res;
                    self.v[0xF] = !overflow as u8;
                }
                0x7 => {
                    let (res, overflow) = self.v[y].overflowing_sub(self.v[x]);
                    self.v[x] = res;
                    self.v[0xF] = !overflow as u8;
                }
                // Shift
                0x6 => {
                    if self.cosmic {
                        self.v[x] = self.v[y];
                    }
                    self.v[0xF] = self.v[x] & 1;
                    self.v[x] >>= 1;
                }
                0xE => {
                    if self.cosmic {
                        self.v[x] = self.v[y];
                    }
                    self.v[0xF] = self.v[x] >> 7 & 1;
                    self.v[x] <<= 1;
                }
                _ => {}
            },
            // 9XY0
            0x9000 => {
                if self.v[x] != self.v[y] {
                    self.pc += 2;
                }
            }
            // ANNN
            0xA000 => {
                self.i = nnn;
            }
            // BNNN
            0xB000 => {
                self.pc = if self.cosmic {
                    nnn + self.v[0] as u16
                } else {
                    nnn + self.v[x] as u16
                };
            }
            // CXNN
            0xC000 => {
                self.v[x] = self.rand_state as u8 & nn;
                self.rand_state ^= self.rand_state << 13;
                self.rand_state ^= self.rand_state >> 7;
                self.rand_state ^= self.rand_state << 17;
            }
            // DXYN
            0xD000 => {
                let vx = self.v[x] as usize % WIDTH;
                let vy = self.v[y] as usize % HEIGHT;

                self.v[0xF] = 0;

                for row in 0..n as usize {
                    if (vy + row) >= HEIGHT {
                        break;
                    }
                    let byte = memory[self.i as usize + row];
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
            0xE000 => match nn {
                0x9E => {
                    if keys_pressed[self.v[x] as usize] {
                        self.pc += 2;
                    }
                }
                0xA1 => {
                    if !keys_pressed[self.v[x] as usize] {
                        self.pc += 2;
                    }
                }
                _ => {}
            },
            0xF000 => match nn {
                0x07 => self.v[x] = *delay,
                0x0A => {
                    for (i, key) in keys_released.iter().enumerate() {
                        if *key {
                            self.v[x] = i as u8;
                            return false;
                        }
                    }
                    self.pc -= 2;
                }
                0x15 => *delay = self.v[x],
                0x18 => *sound = self.v[x],
                0x1E => self.i = self.i.wrapping_add(self.v[x] as u16),
                0x29 => self.i = (self.v[x] & 0xF) as u16 * 5 + 0x50,
                0x33 => {
                    memory[self.i as usize] = self.v[x] / 100;
                    memory[self.i as usize + 1] = self.v[x] / 10 % 10;
                    memory[self.i as usize + 2] = self.v[x] % 10;
                }
                0x55 => {
                    for r in 0..=x {
                        memory[self.i as usize + r] = self.v[r];
                    }
                    if self.cosmic {
                        self.i += x as u16 + 1;
                    }
                }
                0x65 => {
                    for r in 0..=x {
                        self.v[r] = memory[self.i as usize + r];
                    }
                    if self.cosmic {
                        self.i += x as u16 + 1;
                    }
                }
                _ => {}
            },
            _ => {}
        }

        return false;
    }
}
