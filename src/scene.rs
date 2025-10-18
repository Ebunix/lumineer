use std::collections::HashMap;

use artnet_protocol::Output;

use crate::{
    config::SceneConfig,
    dmx::DmxAddress,
    error::Error,
    feature::Feature,
    fixture::Fixture,
    universe::Universe,
};

#[derive(Default)]
pub struct Scene {
    fixtures: Vec<Fixture>,
    universes: Vec<Universe>,
}

enum FinalValue {
    F32(f32),
    U16(u16),
}

impl Scene {
    pub fn from_config(config: &SceneConfig) -> Result<Self, Error> {
        let mut scene = Scene::default();

        let mut fixture_map = HashMap::new();
        for fixture in &config.fixtures {
            let mut channels = Vec::new();
            for feature_config in &fixture.features {
                let feature: Feature = (&feature_config.name).try_into()?;
                let mut channel = if feature_config.wide.unwrap_or_default() {
                    feature.as_channel_16bit(feature_config.offset)
                } else {
                    feature.as_channel(feature_config.offset)
                };
                if let Some(presets) = &feature_config.presets {
                    for preset in presets {
                        channel.add_preset(&preset.name, preset.value);
                    }
                }
                channels.push(channel);
            }

            println!("Scene: Loaded fixture definition for \"{}\"", fixture.name);
            fixture_map.insert(fixture.name.clone(), channels);
        }

        for universe in &config.universes {
            while scene.universes.len() < universe.id as usize {
                scene.universes.push(Universe::new(universe.id - 1));
            }
            for fixture in &universe.fixtures {
                let channel_sources = match fixture_map.get(&fixture.name) {
                    None => {
                        eprintln!("Unknown definition for fixture \"{}\"", fixture.name);
                        continue;
                    }
                    Some(channels) => channels,
                };
                let address = DmxAddress::new(fixture.address - 1);
                let new_fixture = Fixture::new(
                    fixture.id,
                    universe.id - 1,
                    address,
                    channel_sources.clone(),
                );
                println!(
                    "Scene: Added fixture instance for \"{}\" at {}.{:03}",
                    fixture.name, universe.id, fixture.address
                );
                scene.add_fixture(new_fixture);
            }
        }

        Ok(scene)
    }

    pub fn add_fixture(&mut self, fixture: Fixture) {
        self.fixtures.push(fixture);
    }

    pub fn set_feature_value(
        &mut self,
        fixture_id: u16,
        feature: Feature,
        value: &str,
    ) -> Result<(), Error> {
        if let Some(fixture) = self.fixtures.iter().find(|f| f.id == fixture_id)
            && let Some(channel) = fixture.get_channel(feature)
        {
            let final_value = if let Some(preset_value) = channel.get_preset(value) {
                FinalValue::U16(*preset_value)
            } else if value.contains(".") {
                FinalValue::F32(value.parse::<f32>()?)
            } else {
                FinalValue::U16(value.parse::<u16>()?)
            };

            let universe_index = fixture.universe_index as usize;
            if universe_index < self.universes.len() {
                let universe = &mut self.universes[universe_index];
                if channel.is_16bit {
                    match final_value {
                        FinalValue::F32(float) => universe.set_16bit_parameter_float(fixture.address + channel.offset, float),
                        FinalValue::U16(int) => universe.set_16bit_parameter(fixture.address + channel.offset, int),
                    }
                } else {
                    match final_value {
                        FinalValue::F32(float) => universe.set_parameter_float(fixture.address + channel.offset, float),
                        FinalValue::U16(int) => universe.set_parameter(fixture.address + channel.offset, (int & 0xff) as u8),
                    }
                }
            } else {
                eprintln!("Scene: Unknown universe {universe_index}");
            }
        }
        Ok(())
    }

    pub fn iter_output(&self) -> impl Iterator<Item = Output> {
        self.universes.iter().map(|universe| universe.into_output())
    }

    pub fn zero(&mut self) {
        for u in &mut self.universes {
            u.zero();
        }
    }
}
