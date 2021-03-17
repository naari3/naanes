pub struct ControllerInput {
    input: u8,
    shift: usize,
    strobe: bool,
}

impl ControllerInput {
    pub fn new(input: u8) -> Self {
        Self {
            input,
            shift: 0,
            strobe: false,
        }
    }

    pub fn update_input(&mut self, byte: u8) {
        self.input = byte;
    }

    pub fn press_button(&mut self, button: Button) {
        match button {
            Button::A => self.input = self.input | 0x01,
            Button::B => self.input = self.input | 0x02,
            Button::Select => self.input = self.input | 0x04,
            Button::Start => self.input = self.input | 0x08,
            Button::Up => self.input = self.input | 0x10,
            Button::Down => self.input = self.input | 0x20,
            Button::Left => self.input = self.input | 0x40,
            Button::Right => self.input = self.input | 0x80,
        }
    }

    pub fn release_button(&mut self, button: Button) {
        match button {
            Button::A => self.input = self.input & !0x01,
            Button::B => self.input = self.input & !0x02,
            Button::Select => self.input = self.input & !0x04,
            Button::Start => self.input = self.input & !0x08,
            Button::Up => self.input = self.input & !0x10,
            Button::Down => self.input = self.input & !0x20,
            Button::Left => self.input = self.input & !0x40,
            Button::Right => self.input = self.input & !0x80,
        }
    }

    pub fn write_byte(&mut self, byte: u8) {
        if byte & 0b1 == 1 {
            self.shift = 0;
            self.strobe = true;
        } else if byte & 0b1 == 0 {
            self.strobe = false
        }
    }

    pub fn read_byte(&mut self) -> u8 {
        let byte = self.input >> self.shift & 1;
        if !self.strobe {
            self.shift = (self.shift + 1) % 8;
        }
        byte
    }

    pub fn read_byte_without_effect(&mut self) -> u8 {
        self.input >> self.shift
    }
}

pub enum Button {
    A,
    B,
    Select,
    Start,
    Up,
    Down,
    Left,
    Right,
}
