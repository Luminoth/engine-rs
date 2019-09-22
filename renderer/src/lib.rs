mod vulkan;

pub use vulkan::*;

use failure::Error;

pub type Result<T> = std::result::Result<T, Error>;
