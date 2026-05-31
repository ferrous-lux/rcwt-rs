use crate::CcBlock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Ch1F1,
    Ch2F1,
    Ch1F2,
    Ch2F2,
    Cea708,
}

pub struct ChannelClassifier {
    field1_ch: u8,
    field2_ch: u8,
}

impl Default for ChannelClassifier {
    fn default() -> Self {
        Self::new()
    }
}

impl ChannelClassifier {
    pub fn new() -> Self {
        ChannelClassifier {
            field1_ch: 0,
            field2_ch: 0,
        }
    }

    pub fn classify(&mut self, block: &CcBlock) -> Option<Channel> {
        if !block.cc_valid || block.is_null() {
            return None;
        }
        match block.cc_type {
            2 | 3 => Some(Channel::Cea708),
            0 => {
                let b0 = block.cc_data[0] & 0x7f;
                match b0 {
                    0x10..=0x17 => self.field1_ch = 0,
                    0x18..=0x1f => self.field1_ch = 1,
                    0x01 => {
                        let ch = block.cc_data[1] & 0x7f;
                        if ch <= 1 {
                            self.field1_ch = ch;
                        }
                    }
                    _ => {}
                }
                match self.field1_ch {
                    0 => Some(Channel::Ch1F1),
                    _ => Some(Channel::Ch2F1),
                }
            }
            1 => {
                let b0 = block.cc_data[0] & 0x7f;
                match b0 {
                    0x10..=0x17 => self.field2_ch = 0,
                    0x18..=0x1f => self.field2_ch = 1,
                    0x01 => {
                        let ch = block.cc_data[1] & 0x7f;
                        if ch <= 1 {
                            self.field2_ch = ch;
                        }
                    }
                    _ => {}
                }
                match self.field2_ch {
                    0 => Some(Channel::Ch1F2),
                    _ => Some(Channel::Ch2F2),
                }
            }
            _ => None,
        }
    }
}
