use std::{collections::HashMap, ops::{Add, Sub}};

use crate::feature::Feature;

#[derive(Debug, Copy, Clone)]
pub struct DmxAddress(pub Option<u16>);

impl DmxAddress {
    pub fn new(address: u16) -> DmxAddress {
        if address >= 512 {
            DmxAddress(None)
        } else {
            DmxAddress(Some(address))
        }
    }
}

impl Into<DmxAddress> for u16 {
    fn into(self) -> DmxAddress {
        DmxAddress::new(self)
    }
}

impl Add<u16> for DmxAddress {
    type Output = DmxAddress;

    fn add(self, rhs: u16) -> Self::Output {
        match self.0 {
            None => DmxAddress(None),
            Some(i) => {
                let new_index = i.wrapping_add(rhs);
                if new_index >= 512 {
                    DmxAddress(None)
                } else {
                    DmxAddress(Some(new_index))
                }
            }
        }
    }
}
impl Sub<u16> for DmxAddress {
    type Output = DmxAddress;

    fn sub(self, rhs: u16) -> Self::Output {
        match self.0 {
            None => DmxAddress(None),
            Some(i) => {
                let new_index = i.wrapping_sub(rhs);
                if new_index >= 512 {
                    DmxAddress(None)
                } else {
                    DmxAddress(Some(new_index))
                }
            }
        }
    }
}


#[derive(Debug, Clone)]
pub struct Channel {
    pub feature: Feature,
    pub offset: u16,
    pub is_16bit: bool,
    presets: HashMap<String, u16>
}

impl Channel {
    pub fn new(feature: Feature, offset: u16, is_16bit: bool) -> Channel {
        Channel { feature, offset, is_16bit, presets: HashMap::new() }
    }
    pub fn get_preset(&self, name: &str) -> Option<&u16> {
        self.presets.get(name)
    }
    pub fn add_preset(&mut self, name: &str, value: u16) {
        self.presets.insert(name.to_owned(), value);
    }
} 
