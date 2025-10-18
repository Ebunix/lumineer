use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub lumineer: LumineerConfig,
    pub scene: SceneConfig,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LumineerConfig {
    pub passcode: String,
    pub artnet_remote: String,
    pub local_address: String,
    pub websocket_port: u16,
    pub udp_port: u16,
    pub frontend_port: u16,
    pub frontend_root: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SceneConfig {
    pub fixtures: Vec<FixtureConfig>,
    pub universes: Vec<UniverseConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FixtureConfig {
    pub name: String,
    pub features: Vec<FeatureConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FeatureConfig {
    pub name: String,
    pub offset: u16,
    pub wide: Option<bool>,
    pub presets: Option<Vec<FeatureConfigPreset>>,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct FeatureConfigPreset {
    pub name: String,
    pub value: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UniverseConfig {
    pub id: u8,
    pub fixtures: Vec<UniverseFixtureConfig>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UniverseFixtureConfig {
    pub id: u16,
    pub name: String,
    pub address: u16,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArtNetConfig {
    pub remote: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigAddress {
    pub address: String,
    pub port: u16,
}

#[derive(clap::Parser)]
pub struct Args {
    #[arg(short, long, default_value = "lumineer.toml")]
    pub config: String,
}
