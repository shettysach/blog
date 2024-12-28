use std::fs;
use std::io;
use std::path::Path;

pub(crate) fn copy_dir<P>(source: P, destination: P) -> io::Result<()>
where
    P: AsRef<Path>,
{
    if !destination.as_ref().exists() {
        fs::create_dir_all(&destination)?;
    }

    for entry in fs::read_dir(&source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;

        let src_path = entry.path();
        let rel_path = src_path.strip_prefix(source.as_ref()).unwrap();
        let dst_path = destination.as_ref().join(rel_path);

        if file_type.is_dir() {
            copy_dir(&src_path, &dst_path)?;
        } else if file_type.is_file() {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}
