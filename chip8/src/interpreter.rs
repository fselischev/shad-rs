use crate::{
    data::{Address, Nibble, OpCode, RegisterIndex, Word},
    image::Image,
    platform::{Platform, Point, Sprite},
    Error, Key, Result, KEYPAD_LAST,
};

////////////////////////////////////////////////////////////////////////////////

pub const SCREEN_WIDTH: usize = 64;
pub const SCREEN_HEIGHT: usize = 32;

////////////////////////////////////////////////////////////////////////////////

pub const MEM_SIZE: usize = Address::DOMAIN_SIZE;
pub const REG_SIZE: usize = 16;
pub const STACK_SIZE: usize = 16;

struct ProgramCounter(usize);
impl ProgramCounter {
    const STEP: usize = 2;
    fn next(&mut self) {
        self.0 += Self::STEP;
    }

    fn skip(&mut self) {
        self.0 += Self::STEP * 2;
    }
}

pub struct Interpreter<P: Platform> {
    platform: P,
    registers: [u8; REG_SIZE],
    index_register: usize,
    memory: [u8; MEM_SIZE],
    pc: ProgramCounter,
    sp: usize,
    call_stack: [usize; STACK_SIZE],
}

impl<P: Platform> Interpreter<P> {
    pub fn new(image: impl Image, platform: P) -> Self {
        let mut interp = Self {
            registers: [0; REG_SIZE],
            platform,
            index_register: 0,
            memory: [0; MEM_SIZE],
            pc: ProgramCounter(image.entry_point().as_usize()),
            sp: 0,
            call_stack: [0; STACK_SIZE],
        };

        image.load_into_memory(&mut interp.memory);

        interp
    }

    pub fn platform(&self) -> &P {
        &self.platform
    }

    pub fn platform_mut(&mut self) -> &mut P {
        &mut self.platform
    }

    pub fn run_next_instruction(&mut self) -> Result<()> {
        match Operation::try_from(self.extract_opcode()) {
            Ok(operation) => {
                match operation {
                    // Test 1: Chip logo
                    Operation::ClearScreen => self.cls(),
                    Operation::Jump(addr) => self.jmp(addr),
                    Operation::SetRegister(vx, nn) => self.set_reg(vx, nn),
                    Operation::SetIndexRegister(addr) => self.set_i(addr),
                    Operation::Draw(vx, vy, n) => self.draw(vx, vy, n),
                    // Test 2: IBM logo
                    Operation::AddValue(vx, nn) => self.add_value(vx, nn),
                    // Test 3, 4: Corax, Flags
                    Operation::SkipIfEqual(vx, nn) => self.skip_if_eq(vx, nn),
                    Operation::SkipIfNotEqual(vx, nn) => self.skip_if_neq(vx, nn),
                    Operation::SkipIfRegistersEqual(vx, vy) => self.skip_if_reg_eq(vx, vy),
                    Operation::SkipIfRegistersNotEqual(vx, vy) => self.skip_if_reg_neq(vx, vy),
                    Operation::Call(nnn) => self.call(nnn)?,
                    Operation::Return => self.ret()?,
                    Operation::SetToRegister(vx, vy) => self.set_to_reg(vx, vy),
                    Operation::Or(vx, vy) => self.or(vx, vy),
                    Operation::And(vx, vy) => self.and(vx, vy),
                    Operation::Xor(vx, vy) => self.xor(vx, vy),
                    Operation::AddRegister(vx, vy) => self.add_to_reg(vx, vy),
                    Operation::SubRegister(vx, vy) => self.sub(vx, vy),
                    Operation::SubRegisterReversed(vx, vy) => self.sub_rev(vx, vy),
                    Operation::ShiftRight(vx, vy) => self.shr(vx, vy),
                    Operation::ShiftLeft(vx, vy) => self.shl(vx, vy),
                    Operation::ReadMemory(vx) => self.read(vx),
                    Operation::WriteMemory(vx) => self.write(vx),
                    Operation::ToDecimal(vx) => self.dec(vx),
                    Operation::IncrementIndexRegister(vx) => self.incr_i(vx),
                    // Test 5: Quirks
                    Operation::SkipIfKeyDown(vx) => self.key_down(vx)?,
                    Operation::SkipIfKeyUp(vx) => self.key_up(vx)?,
                    Operation::SetDelayTimer(vx) => self.set_delay_timer(vx),
                    Operation::GetDelayTimer(vx) => self.get_delay_timer(vx),
                    Operation::SetSoundTimer(vx) => self.set_sound_timer(vx),
                    Operation::JumpV0(nnn) => self.jmp_v0(nnn),
                    // Test 6: Keypad
                    Operation::WaitForKey(vx) => self.wait_for_key(vx),
                    // other
                    Operation::SetToRandom(vx, nn) => self.set_rng(vx, nn),
                    Operation::SetIndexRegisterToSprite(vx) => self.set_sprite(vx),
                }
                Ok(())
            }
            Err(_) => Err(Error::Crashed),
        }
    }

    fn extract_opcode(&self) -> OpCode {
        OpCode::new((self.memory[self.pc.0] as u16) << 8 | (self.memory[self.pc.0 + 1] as u16))
    }
}

impl<P: Platform> Interpreter<P> {
    // 00E0
    fn cls(&mut self) {
        self.platform.clear_screen();
        self.pc.next();
    }

    // 6xnn
    fn set_reg(&mut self, x: RegisterIndex, nn: Word) {
        self.registers[x.as_usize()] = nn;
        self.pc.next();
    }

    // Annn
    fn set_i(&mut self, addr: Address) {
        self.index_register = addr.as_usize();
        self.pc.next();
    }

    // 1nnn
    fn jmp(&mut self, nnn: Address) {
        self.pc.0 = nnn.as_usize();
    }

    // Dxyn
    fn draw(&mut self, x: RegisterIndex, y: RegisterIndex, n: Nibble) {
        self.registers[0x0f] = if self.platform.draw_sprite(
            Point(self.registers[x.as_usize()], self.registers[y.as_usize()]),
            Sprite::new(&self.memory[self.index_register..self.index_register + n.as_usize()]),
        ) {
            1
        } else {
            0
        };

        self.pc.next();
    }

    // 7xnn
    fn add_value(&mut self, x: RegisterIndex, nn: Word) {
        self.registers[x.as_usize()] = self.registers[x.as_usize()].wrapping_add(nn);
        self.pc.next();
    }

    // 3xnn
    fn skip_if_eq(&mut self, x: RegisterIndex, nn: Word) {
        if self.registers[x.as_usize()] == nn {
            self.pc.skip();
        } else {
            self.pc.next();
        }
    }

    // 4xnn
    fn skip_if_neq(&mut self, x: RegisterIndex, nn: Word) {
        if self.registers[x.as_usize()] != nn {
            self.pc.skip();
        } else {
            self.pc.next();
        }
    }

    // 5xy0
    fn skip_if_reg_eq(&mut self, x: RegisterIndex, y: RegisterIndex) {
        if self.registers[x.as_usize()] == self.registers[y.as_usize()] {
            self.pc.skip();
        } else {
            self.pc.next();
        }
    }

    // 9xy0
    fn skip_if_reg_neq(&mut self, x: RegisterIndex, y: RegisterIndex) {
        if self.registers[x.as_usize()] != self.registers[y.as_usize()] {
            self.pc.skip();
        } else {
            self.pc.next();
        }
    }

    // 2nnn
    fn call(&mut self, nnn: Address) -> Result<()> {
        let (res, overflow) = self.sp.overflowing_add(1);
        if overflow {
            Err(Error::StackOverflow)
        } else {
            self.call_stack[self.sp] = self.pc.0 + 2;
            self.sp = res;
            self.pc.0 = nnn.as_usize();
            Ok(())
        }
    }

    // 00EE
    fn ret(&mut self) -> Result<()> {
        let (res, underflow) = self.sp.overflowing_sub(1);
        if underflow {
            Err(Error::StackUnderflow)
        } else {
            self.sp = res;
            self.pc.0 = self.call_stack[self.sp];
            Ok(())
        }
    }

    // 8xy0
    fn set_to_reg(&mut self, x: RegisterIndex, y: RegisterIndex) {
        self.registers[x.as_usize()] = self.registers[y.as_usize()];
        self.pc.next();
    }

    // 8xy1
    fn or(&mut self, x: RegisterIndex, y: RegisterIndex) {
        self.registers[x.as_usize()] |= self.registers[y.as_usize()];
        self.registers[0x0f] = 0;
        self.pc.next();
    }

    // 8xy2
    fn and(&mut self, x: RegisterIndex, y: RegisterIndex) {
        self.registers[x.as_usize()] &= self.registers[y.as_usize()];
        self.registers[0x0f] = 0;
        self.pc.next();
    }

    // 8xy3
    fn xor(&mut self, x: RegisterIndex, y: RegisterIndex) {
        self.registers[x.as_usize()] ^= self.registers[y.as_usize()];
        self.registers[0x0f] = 0;
        self.pc.next();
    }

    // 8xy4
    fn add_to_reg(&mut self, x: RegisterIndex, y: RegisterIndex) {
        let (res, overflow) =
            self.registers[x.as_usize()].overflowing_add(self.registers[y.as_usize()]);
        self.registers[x.as_usize()] = res;
        self.registers[0x0f] = if overflow { 1 } else { 0 };

        self.pc.next();
    }

    // 8xy5
    fn sub(&mut self, x: RegisterIndex, y: RegisterIndex) {
        let (res, underflow) =
            self.registers[x.as_usize()].overflowing_sub(self.registers[y.as_usize()]);
        self.registers[x.as_usize()] = res;

        self.registers[0x0f] = if underflow { 0 } else { 1 };

        self.pc.next();
    }

    // 8xy7
    fn sub_rev(&mut self, x: RegisterIndex, y: RegisterIndex) {
        let (res, underflow) =
            self.registers[y.as_usize()].overflowing_sub(self.registers[x.as_usize()]);
        self.registers[x.as_usize()] = res;

        self.registers[0x0f] = if underflow { 0 } else { 1 };

        self.pc.next();
    }

    // 8xy6
    fn shr(&mut self, x: RegisterIndex, y: RegisterIndex) {
        let vy = self.registers[y.as_usize()];
        self.registers[x.as_usize()] = self.registers[y.as_usize()] >> 1;
        self.registers[0x0f] = vy & 0x1;
        self.pc.next();
    }

    // 8xyE
    fn shl(&mut self, x: RegisterIndex, y: RegisterIndex) {
        let vy = self.registers[y.as_usize()];
        self.registers[x.as_usize()] = self.registers[y.as_usize()] << 1;
        self.registers[0x0f] = vy >> 7;
        self.pc.next();
    }

    // Fx65
    fn read(&mut self, x: Nibble) {
        for i in 0..x.as_usize() + 1 {
            self.registers[i] = self.memory[self.index_register + i];
        }
        self.index_register += x.as_usize() + 1;
        self.pc.next();
    }

    // Fx55
    fn write(&mut self, x: Nibble) {
        for i in 0..x.as_usize() + 1 {
            self.memory[self.index_register + i] = self.registers[i];
        }
        self.index_register += x.as_usize() + 1;
        self.pc.next();
    }

    // Fx33
    fn dec(&mut self, x: RegisterIndex) {
        self.memory[self.index_register] = self.registers[x.as_usize()] / 100;
        self.memory[self.index_register + 1] = (self.registers[x.as_usize()] % 100) / 10;
        self.memory[self.index_register + 2] = self.registers[x.as_usize()] % 10;
        self.pc.next();
    }

    // Fx1E
    fn incr_i(&mut self, x: RegisterIndex) {
        self.index_register += self.registers[x.as_usize()] as usize;
        self.pc.next();
    }

    // Ex9E
    fn key_down(&mut self, x: Key) -> Result<()> {
        let key = Nibble::try_from(self.registers[x.as_usize()]).unwrap();
        match key.as_u8() {
            0..=KEYPAD_LAST => {
                if self.platform.is_key_down(key) {
                    self.pc.skip();
                } else {
                    self.pc.next();
                }
                Ok(())
            }
            _ => Err(Error::InvalidKey(key.as_u8())),
        }
    }

    // ExA1
    fn key_up(&mut self, x: Key) -> Result<()> {
        let key = Nibble::try_from(self.registers[x.as_usize()]).unwrap();
        match key.as_u8() {
            0..=KEYPAD_LAST => {
                if !self.platform.is_key_down(key) {
                    self.pc.skip();
                } else {
                    self.pc.next();
                }
                Ok(())
            }
            _ => Err(Error::InvalidKey(key.as_u8())),
        }
    }

    // Fx15
    fn set_delay_timer(&mut self, x: Nibble) {
        self.platform.set_delay_timer(self.registers[x.as_usize()]);
        self.pc.next();
    }

    // Fx07
    fn get_delay_timer(&mut self, x: Nibble) {
        self.registers[x.as_usize()] = self.platform.get_delay_timer();
        self.pc.next();
    }

    // Fx18
    fn set_sound_timer(&mut self, x: Nibble) {
        self.platform.set_sound_timer(x.as_u8());
        self.pc.next();
    }

    // Fx0A
    fn wait_for_key(&mut self, x: Nibble) {
        let mut fl = false;
        for i in 0..16 {
            if self.platform.is_key_down(Nibble::try_from(i).unwrap()) {
                self.memory[x.as_usize()] = i;
                fl = true;
            }
        }

        if !fl {
            return;
        }

        self.pc.next();
    }

    // Bnnn
    fn jmp_v0(&mut self, nnn: Address) {
        self.pc.0 = (nnn + self.registers[0] as i16).as_usize();
    }

    // Cxnn
    fn set_rng(&mut self, x: RegisterIndex, nn: Word) {
        self.registers[x.as_usize()] = self.platform_mut().get_random_word() & nn;
        self.pc.next();
    }

    // Fx29
    fn set_sprite(&mut self, x: Nibble) {
        self.index_register = (self.registers[x.as_usize()] as usize) * 5;
        self.pc.next();
    }
}

////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone, Copy)]
pub enum Operation {
    ClearScreen,
    Return,
    Jump(Address),
    Call(Address),
    SkipIfEqual(RegisterIndex, Word),
    SkipIfNotEqual(RegisterIndex, Word),
    SkipIfRegistersEqual(RegisterIndex, RegisterIndex),
    SetRegister(RegisterIndex, Word),
    AddValue(RegisterIndex, Word),
    SetToRegister(RegisterIndex, RegisterIndex),
    Or(RegisterIndex, RegisterIndex),
    And(RegisterIndex, RegisterIndex),
    Xor(RegisterIndex, RegisterIndex),
    AddRegister(RegisterIndex, RegisterIndex),
    SubRegister(RegisterIndex, RegisterIndex),
    ShiftRight(RegisterIndex, RegisterIndex),
    SubRegisterReversed(RegisterIndex, RegisterIndex),
    ShiftLeft(RegisterIndex, RegisterIndex),
    SkipIfRegistersNotEqual(RegisterIndex, RegisterIndex),
    SetIndexRegister(Address),
    JumpV0(Address),
    SetToRandom(RegisterIndex, Word),
    Draw(RegisterIndex, RegisterIndex, Nibble),
    SkipIfKeyDown(RegisterIndex),
    SkipIfKeyUp(RegisterIndex),
    GetDelayTimer(RegisterIndex),
    WaitForKey(RegisterIndex),
    SetDelayTimer(RegisterIndex),
    SetSoundTimer(RegisterIndex),
    IncrementIndexRegister(RegisterIndex),
    SetIndexRegisterToSprite(Nibble),
    ToDecimal(RegisterIndex),
    WriteMemory(Nibble),
    ReadMemory(Nibble),
}

impl TryFrom<OpCode> for Operation {
    type Error = Error;

    fn try_from(code: OpCode) -> std::result::Result<Self, Self::Error> {
        let nibbles = (0..4)
            .map(|i| code.extract_nibble(i).as_u8())
            .rev()
            .collect::<Vec<_>>();
        let nn = code.extract_word(0);
        let nnn = code.extract_address();

        let op = match nibbles.as_slice() {
            // Test 1: Chip logo
            [0x00, 0x00, 0x0e, 0x00] => Self::ClearScreen,
            [0x06, x, ..] => Self::SetRegister(RegisterIndex::try_from(*x)?, nn),
            [0x0A, ..] => Self::SetIndexRegister(nnn),
            [0x01, ..] => Self::Jump(nnn),
            [0x0D, ..] => Self::Draw(
                RegisterIndex::try_from(nibbles[1])?,
                RegisterIndex::try_from(nibbles[2])?,
                Nibble::try_from(nibbles[3])?,
            ),
            // Test 2: IBM logo
            [0x07, x, ..] => Self::AddValue(RegisterIndex::try_from(*x)?, nn),
            // Test 3, 4: Corax, Flags
            [0x03, x, ..] => Self::SkipIfEqual(RegisterIndex::try_from(*x)?, nn),
            [0x04, x, ..] => Self::SkipIfNotEqual(RegisterIndex::try_from(*x)?, nn),
            [0x05, x, y, 0x00] => Self::SkipIfRegistersEqual(
                RegisterIndex::try_from(*x)?,
                RegisterIndex::try_from(*y)?,
            ),
            [0x09, x, y, 0x00] => Self::SkipIfRegistersNotEqual(
                RegisterIndex::try_from(*x)?,
                RegisterIndex::try_from(*y)?,
            ),
            [0x02, ..] => Self::Call(nnn),
            [0x00, 0x00, 0x0e, 0x0e] => Self::Return,
            [0x08, x, y, 0x00] => {
                Self::SetToRegister(RegisterIndex::try_from(*x)?, RegisterIndex::try_from(*y)?)
            }
            [0x08, x, y, 0x01] => {
                Self::Or(RegisterIndex::try_from(*x)?, RegisterIndex::try_from(*y)?)
            }
            [0x08, x, y, 0x02] => {
                Self::And(RegisterIndex::try_from(*x)?, RegisterIndex::try_from(*y)?)
            }
            [0x08, x, y, 0x03] => {
                Self::Xor(RegisterIndex::try_from(*x)?, RegisterIndex::try_from(*y)?)
            }
            [0x08, x, y, 0x04] => {
                Self::AddRegister(RegisterIndex::try_from(*x)?, RegisterIndex::try_from(*y)?)
            }
            [0x08, x, y, 0x05] => {
                Self::SubRegister(RegisterIndex::try_from(*x)?, RegisterIndex::try_from(*y)?)
            }
            [0x08, x, y, 0x07] => Self::SubRegisterReversed(
                RegisterIndex::try_from(*x)?,
                RegisterIndex::try_from(*y)?,
            ),
            [0x08, x, y, 0x06] => {
                Self::ShiftRight(RegisterIndex::try_from(*x)?, RegisterIndex::try_from(*y)?)
            }
            [0x08, x, y, 0x0e] => {
                Self::ShiftLeft(RegisterIndex::try_from(*x)?, RegisterIndex::try_from(*y)?)
            }
            [0x0f, x, 0x06, 0x05] => Self::ReadMemory(Nibble::try_from(*x)?),
            [0x0f, x, 0x05, 0x05] => Self::WriteMemory(Nibble::try_from(*x)?),
            [0x0f, x, 0x03, 0x03] => Self::ToDecimal(RegisterIndex::try_from(*x)?),
            [0x0f, x, 0x01, 0x0e] => Self::IncrementIndexRegister(RegisterIndex::try_from(*x)?),
            [0x0e, x, 0x09, 0x0e] => Self::SkipIfKeyDown(Nibble::try_from(*x)?),
            [0x0e, x, 0x0a, 0x01] => Self::SkipIfKeyUp(Nibble::try_from(*x)?),
            // Test 5: Quirks
            [0x0f, x, 0x01, 0x05] => Self::SetDelayTimer(RegisterIndex::try_from(*x)?),
            [0x0f, x, 0x00, 0x07] => Self::GetDelayTimer(Nibble::try_from(*x)?),
            [0x0f, x, 0x01, 0x08] => Self::SetSoundTimer(Nibble::try_from(*x)?),
            [0x0b, ..] => Self::JumpV0(nnn),
            // Test 6: Keypad
            [0x0f, x, 0x00, 0x0a] => Self::WaitForKey(Nibble::try_from(*x)?),
            // other
            [0x0c, x, ..] => Self::SetToRandom(Nibble::try_from(*x)?, nn),
            [0x0f, x, 0x02, 0x09] => Self::SetIndexRegisterToSprite(Nibble::try_from(*x)?),
            _ => return Err(Error::UnknownOpCode(code)),
        };
        Ok(op)
    }
}

////////////////////////////////////////////////////////////////////////////////
