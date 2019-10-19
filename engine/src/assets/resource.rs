use std::path::Path;

use log::warn;

pub trait Resource {
    const EXTENSION: &'static str;

    fn load<P>(filepath: P) -> anyhow::Result<()>
    where
        P: AsRef<Path>,
    {
        warn!("TODO: load resource at {}", filepath.as_ref().display());

        Ok(())
    }
}
