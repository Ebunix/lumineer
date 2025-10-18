use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{error::Error, feature::{ControlFeature, Feature}, scene::Scene};


pub type MessageData = (u16, Feature, String);

pub async fn handle_incoming_data<T>(scene: Arc<RwLock<Scene>>, data: T) -> Result<bool, Error>
where
    T: AsRef<[u8]>,
{
    let text = match String::from_utf8(data.as_ref().to_vec()) {
        Ok(message) => message,
        Err(error) => {
            eprintln!("Failed to parse message: {}", error);
            return Err(Error::FormatError);
        }
    };
    let packets: Vec<MessageData> = text
        .split("\n")
        .filter_map(|line| {
            let mut parts = line.trim().split(" ");
            if let (Some(fixture_id), Some(feature_id), Some(value)) =
                (parts.next(), parts.next(), parts.next())
            {
                let feature: Option<Feature> = feature_id.try_into().ok();
                if let Some(feature) = feature
                    && let Ok(fixture_id) = u16::from_str_radix(fixture_id, 10)
                {
                    return Some((fixture_id, feature, value.to_owned()));
                }
                eprintln!("Parse: Failed on \"{}\"", line);
            }
            return None;
        })
        .collect();

    {
        let mut lock = scene.write().await;

        for (fixture_id, feature, value) in packets {
            if feature == Feature::Control {
                match TryInto::<ControlFeature>::try_into(&value) {
                    Ok(ControlFeature::Zero) => {
                        lock.zero();
                    }
                    Ok(ControlFeature::End) => {
                        lock.zero();
                        return Ok(false);
                    }
                    Err(error) => {
                        eprintln!("Control: {error}");
                    }
                }
            } else {
                lock.set_feature_value(fixture_id, feature, &value).ok();
            }
        }
    }

    Ok(true)
}