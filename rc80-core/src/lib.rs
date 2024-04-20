pub const MEMORY_SIZE: usize = 4096;
pub const REGISTER_AMOUNT: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;
pub const PIXEL_AMOUNT: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

pub type ScreenPixels = [u8; PIXEL_AMOUNT];

pub struct System {
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub stack: [u16; STACK_SIZE],
    pub memory: [u8; MEMORY_SIZE],
    pub v_registers: [u8; REGISTER_AMOUNT],
    pub i_register: u16,
    pub time_register: u8,
    pub sound_register: u8,
    pub pixels: ScreenPixels,
}

impl Default for System {
    fn default() -> Self {
        Self {
            program_counter: 0x200,
            stack_pointer: 0,
            stack: [0; STACK_SIZE],
            memory: [0; MEMORY_SIZE],
            v_registers: [0; REGISTER_AMOUNT],
            i_register: 0,
            time_register: 0,
            sound_register: 0,
            pixels: [0; PIXEL_AMOUNT],
        }
    }
}

impl System {
    pub fn load(&mut self, program_bytes: &[u8]) {
        println!("Memory length: {}", self.memory.len());
        println!("Program length: {}", program_bytes.len());
        let len = program_bytes.len() + 0x200;
        println!("Calculated destination length: {}", len);
        self.memory[0x200..len].copy_from_slice(program_bytes);
    }

    pub fn step(&mut self) {
        let opcode = (self.memory[self.program_counter as usize] as u16) << 8
            | self.memory[self.program_counter as usize + 1] as u16;
        println!("Found opcode: {:#06x}", opcode);
        let code_type = opcode >> 12;
        let mut jumped = false;
        match code_type {
            0x0 => match opcode {
                0x00E0 => {
                    println!("Clear screen!");
                    self.clear_screen();
                }
                0x00EE => {
                    println!("Return from subroutine");
                    self.stack_pointer -= 1;
                    self.program_counter = self.stack[self.stack_pointer as usize];
                    self.stack[self.stack_pointer as usize] = 0;
                }
                _ => {
                    println!("not supported");
                }
            },
            0x1 => {
                let target = opcode & 0x0FFF;
                println!("Jump to {:#x}", target);
                self.program_counter = target;
                jumped = true;
            }
            0x2 => {
                let target = opcode & 0x0FFF;
                println!("Call subroutine {:x}", target);
                self.stack[self.stack_pointer as usize] = self.program_counter;
                self.stack_pointer += 1;
                self.program_counter = target;
                jumped = true;
            }
            0x3 => {
                let register = (opcode >> 8 & 0xF) as usize;
                let value = (opcode & 0xFF) as u8;
                println!("Register V{:x} == {}?", register, value);
                if self.v_registers[register] == value {
                    println!("Skipped line");
                    self.program_counter += 2;
                }
            }
            0x4 => {
                let register = (opcode >> 8 & 0xF) as usize;
                let value = (opcode & 0xFF) as u8;
                println!("Register V{:x} != {}?", register, value);
                if self.v_registers[register] != value {
                    println!("Skipped line");
                    self.program_counter += 2;
                }
            }
            0x5 => {
                let register1 = (opcode >> 8 & 0xF) as usize;
                let register2 = (opcode >> 4 & 0xF) as usize;
                println!("Register V{:x} == V{:x}?", register1, register2);
                if self.v_registers[register1] == self.v_registers[register2] {
                    println!("Skipped line");
                    self.program_counter += 2;
                }
            }
            0x6 => {
                let register_index = (opcode >> 8 & 0xF) as usize;
                let register_value = (opcode & 0xFF) as u8;
                println!("Set register V{:x} to {}", register_index, register_value);
                self.v_registers[register_index] = register_value;
            }
            0x7 => {
                let register_index = (opcode >> 8 & 0xF) as usize;
                let value = (opcode & 0xFF) as u8;
                println!("Increment register V{:x} by {}", register_index, value);
                let reg = &mut self.v_registers[register_index];
                *reg = reg.overflowing_add(value).0;
            }
            0x8 => {
                let register1 = (opcode >> 8 & 0xF) as usize;
                let register2 = (opcode >> 4 & 0xF) as usize;
                let op = opcode & 0xF;
                let reg2 = self.v_registers[register2];
                let reg1 = &mut self.v_registers[register1];
                match op {
                    0x0 => {
                        println!("Operation V{:x} = V{:x}", register1, register2);
                        *reg1 = reg2;
                    }
                    0x1 => {
                        println!("Operation V{:x} |= V{:x}", register1, register2);
                        *reg1 |= reg2;
                    }
                    0x2 => {
                        println!("Operation V{:x} &= V{:x}", register1, register2);
                        *reg1 &= reg2;
                    }
                    0x3 => {
                        println!("Operation V{:x} ^= V{:x}", register1, register2);
                        *reg1 ^= reg2;
                    }
                    0x4 => {
                        println!("Operation V{:x} += V{:x}", register1, register2);
                        let res = reg1.overflowing_add(reg2);
                        *reg1 = res.0;
                        self.v_registers[15] = res.1 as u8;
                    }
                    0x5 => {
                        println!("Operation V{:x} -= V{:x}", register1, register2);
                        let res = reg1.overflowing_sub(reg2);
                        *reg1 = res.0;
                        self.v_registers[15] = res.1 as u8;
                    }
                    0x6 => {
                        println!("Operation V{:x} >> 1", register1);
                        let res = reg1.overflowing_shr(1);
                        *reg1 = res.0;
                        self.v_registers[15] = res.1 as u8;
                    }
                    0x7 => {
                        println!(
                            "Operation V{:x} = V{:x} - V{:x}",
                            register1, register2, register1
                        );
                        let res = reg2.overflowing_sub(*reg1);
                        *reg1 = res.0;
                        self.v_registers[15] = res.1 as u8;
                    }
                    0xE => {
                        println!("Operation V{:x} << 1", register1);
                        let res = reg1.overflowing_shl(1);
                        *reg1 = res.0;
                        self.v_registers[15] = res.1 as u8;
                    }
                    _ => {
                        println!("what the hell??");
                    }
                }
            }
            0x9 => {
                let register1 = (opcode >> 8 & 0xF) as usize;
                let register2 = (opcode >> 4 & 0xF) as usize;
                println!("Register V{:x} != V{:x}?", register1, register2);
                if self.v_registers[register1] != self.v_registers[register2] {
                    println!("Skipped line");
                    self.program_counter += 2;
                }
            }
            0xA => {
                let register_value = opcode & 0x0FFF;
                println!("Set register I to {:#x}", register_value);
                self.i_register = register_value;
            }
            0xD => {
                let vx_reg = opcode >> 8 & 0xF;
                let vy_reg = opcode >> 4 & 0xF;
                let x = self.v_registers[vx_reg as usize];
                let y = self.v_registers[vy_reg as usize];
                let data_size = (opcode & 0xF) as usize;
                println!("Draw sprite at {:?} with size {}", (x, y), data_size);
                let mut bytes = vec![];
                for i in 0..data_size {
                    bytes.push(self.memory[self.i_register as usize + i]);
                }
                let mut pixel_data = vec![];
                for byte in bytes {
                    for b in (0..u8::BITS).rev() {
                        let b = byte >> b & 1;
                        pixel_data.push(b);
                    }
                }
                self.blit_sprite(x, y, &pixel_data);
            }
            0xF => {
                let reg_val = opcode >> 8 & 0xF;
                let code = opcode & 0xFF;
                match code {
                    0x1E => {
                        let val = self.v_registers[reg_val as usize];
                        println!("Incrementing I by V{:x} = {}", reg_val, val);
                        self.i_register += val as u16;
                    }
                    0x33 => {
                        println!("Saving BCD value of register V{:x}", reg_val);
                        let val = format!("{:0>3}", self.v_registers[reg_val as usize]);
                        println!("{:?}", val);
                        val.as_bytes().iter().enumerate().for_each(|(i, v)| {
                            println!("{} - {:b}", v, v);
                            self.memory[self.i_register as usize + i] = v & 0xF;
                        });
                    }
                    0x55 => {
                        let reg_index = reg_val as usize;
                        for i in 0..=reg_index {
                            let val = self.v_registers[i];
                            self.memory[self.i_register as usize] = val;
                            println!(
                                "Saving value of register V{:x} to address {:x}",
                                i, self.i_register
                            );
                            self.i_register += 1;
                        }
                    }
                    0x65 => {
                        let reg_index = reg_val as usize;
                        for i in 0..=reg_index {
                            let val = self.memory[self.i_register as usize];
                            self.v_registers[i] = val;
                            self.i_register += 1;
                            println!("Loading value for register V{:x} = {}", i, val);
                        }
                    }
                    _ => {
                        println!("unimplemented");
                    }
                }
            }
            _ => {
                println!("unimplemented");
            }
        }
        if !jumped {
            self.program_counter += 2;
        }
    }

    fn clear_screen(&mut self) {
        self.pixels.iter_mut().for_each(|p| *p = 0);
    }

    fn blit_sprite(&mut self, x: u8, y: u8, sprite: &[u8]) {
        self.v_registers[15] = 0;
        for (index, pixel) in sprite.iter().enumerate() {
            if *pixel == 1 {
                let mut x = x as usize + index % 8;
                let mut y = y as usize + index / 8;
                if x >= SCREEN_WIDTH {
                    x -= SCREEN_WIDTH;
                }
                if y >= SCREEN_HEIGHT {
                    y -= SCREEN_HEIGHT;
                }
                let target = &mut self.pixels[y * SCREEN_WIDTH + x];
                if *target == 1 {
                    self.v_registers[15] = 1;
                }
                *target ^= *pixel;
            }
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_system() {}
}
