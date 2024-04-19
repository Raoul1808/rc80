pub const MEMORY_SIZE: usize = 4096;
pub const REGISTER_AMOUNT: usize = 16;
pub const STACK_SIZE: usize = 16;
pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

pub struct System {
    pub program_counter: u16,
    pub stack_pointer: u8,
    pub stack: [u16; STACK_SIZE],
    pub memory: [u8; MEMORY_SIZE],
    pub v_registers: [u8; REGISTER_AMOUNT],
    pub i_register: u16,
    pub time_register: u8,
    pub sound_register: u8,
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
        self.program_counter += 2;
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_system() {}
}
