use std::{
    io::Write,
    ops::{Index, IndexMut},
};

use artnet_protocol::{Output, PortAddress};

use crate::dmx::DmxAddress;

pub struct Universe {
    index: u8,
    data: [u8; 512],
    invalid_address_buffer: u8,
}

impl Universe {
    pub fn new(index: u8) -> Universe {
        Universe {
            index,
            data: [0; _],
            invalid_address_buffer: 0,
        }
    }
    pub fn into_output(&self) -> Output {
        let mut out = Output::default();
        out.port_address = PortAddress::from(self.index);
        out.data.as_mut().write_all(&self.data).ok();
        out
    }
    pub fn zero(&mut self) {
        self.data.fill(0);
    }
    pub fn set_parameter(&mut self, parameter: DmxAddress, value: u8) {
        self[parameter] = value;
    }
    pub fn set_parameter_float(&mut self, parameter: DmxAddress, value: f32) {
        self.set_parameter(parameter, (value * 255.0).round() as u8);
    }
    pub fn set_16bit_parameter(&mut self, parameter: DmxAddress, value: u16) {
        self[parameter] = ((value & 0xff00) >> 8) as u8;
        self[parameter + 1] = (value & 0xff) as u8;
    }
    pub fn set_16bit_parameter_float(&mut self, parameter: DmxAddress, value: f32) {
        self.set_16bit_parameter(parameter, (value * 65535.0).round() as u16);
    }
}

impl Index<DmxAddress> for Universe {
    type Output = u8;

    fn index(&self, index: DmxAddress) -> &Self::Output {
        match index.0 {
            None => &self.invalid_address_buffer,
            Some(i) => &self.data[i as usize],
        }
    }
}
impl IndexMut<DmxAddress> for Universe {
    fn index_mut(&mut self, index: DmxAddress) -> &mut Self::Output {
        match index.0 {
            None => &mut self.invalid_address_buffer,
            Some(i) => &mut self.data[i as usize],
        }
    }
}
