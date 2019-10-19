use std::path::Path;

use log::warn;

pub trait Resource {
    const EXTENSION: &'static str;
}
