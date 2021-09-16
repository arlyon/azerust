use bincode::{config, DefaultOptions, Options};

pub fn wow_bincode() -> config::WithOtherStrEncoding<
    config::WithOtherIntEncoding<DefaultOptions, config::FixintEncoding>,
    config::NullTerminatedStrEncoding,
> {
    DefaultOptions::new()
        .with_fixint_encoding()
        .with_null_terminated_str_encoding()
}
