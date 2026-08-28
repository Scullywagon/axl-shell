pub const NEW_LINE: u8 = b'\n';
pub const RETURN: u8 = b'\r';
pub const END_OF_TEXT: u8 = 0x03;
pub const END_OF_FILE: u8 = 0x04;
pub const BACKSPACE: u8 = 0x7F;
pub const ESCAPE: u8 = 0x1B;

pub const UP: [u8; 2] = [b'[', b'A'];
pub const DOWN: [u8; 2] = [b'[', b'B'];
pub const LEFT: [u8; 2] = [b'[', b'D'];
pub const RIGHT: [u8; 2] = [b'[', b'C'];

pub const DELETE: [u8; 3] = [b'[', b'3', b'~'];
