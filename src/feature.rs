use crate::dmx::Channel;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Feature {
    Control,
    Dim,
    Pan,
    Tilt,
    ColorWheel,
    ColorR,
    ColorG,
    ColorB,
    Gobo1,
    Effect,
    Prism,
    Shutter,
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum ControlFeature {
    Zero,
    End,
}

impl Feature {
    pub fn as_channel(self, offset: u16) -> Channel {
        Channel::new(self, offset, false)
    }
    pub fn as_channel_16bit(self, offset: u16) -> Channel {
        Channel::new(self, offset, true)
    }
}

macro_rules! impl_enum_from_str {
    ($enum:ident, $($name:ident),* $(,)?) => {
        impl TryInto<$enum> for &str {
            type Error = $crate::error::Error;
            fn try_into(self) -> Result<$enum, Self::Error> {
                let name = self.as_ref();
                match name {
                    $(stringify!($name) => Ok($enum::$name),)*
                    _ => Err($crate::error::Error::InvalidFeatureName(name.to_owned())),
                }
            }
        }
        impl TryInto<$enum> for &String {
            type Error = $crate::error::Error;
            fn try_into(self) -> Result<$enum, Self::Error> {
                let name = self.as_ref();
                match name {
                    $(stringify!($name) => Ok($enum::$name),)*
                    _ => Err($crate::error::Error::InvalidFeatureName(name.to_owned())),
                }
            }
        }
    };
}

impl_enum_from_str! {
    Feature,
    Control,
    Dim,
    Pan,
    Tilt,
    ColorWheel,
    ColorR,
    ColorG,
    ColorB,
    Gobo1,
    Effect,
    Prism,
    Shutter,
}

impl_enum_from_str! {
    ControlFeature,
    Zero,
    End
}
