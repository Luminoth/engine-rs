use std::path::{Path, PathBuf};

pub fn to_absolute_path<P>(filepath: P) -> anyhow::Result<PathBuf>
where
    P: AsRef<Path>,
{
    let filepath = filepath.as_ref();
    Ok(if filepath.is_absolute() {
        filepath.to_path_buf()
    } else {
        let mut scratch = PathBuf::new();
        scratch.push(std::env::current_dir()?);
        scratch.push(filepath);
        scratch
    })
}
