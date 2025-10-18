use crate::{dmx::{Channel, DmxAddress}, feature::Feature};

pub struct Fixture {
    pub id: u16,
    pub universe_index: u8,
    pub address: DmxAddress,
    pub channels: Vec<Channel>,
}

impl Fixture {
    pub fn new(id: u16, universe: u8, address: DmxAddress, channels: Vec<Channel>) -> Self {
        Self {
            id,
            universe_index: universe,
            address,
            channels,
        }
    }
    pub fn get_channel(&self, feature: Feature) -> Option<&Channel> {
        self.channels.iter().find(|c| c.feature == feature)
    }
}
