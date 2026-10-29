use std::path::Path;

pub fn can_read(path: &Path) -> bool {
    match std::fs::metadata(path) {
        Ok(_metadata) => {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = _metadata.permissions().mode();
                mode & 0o444 != 0
            }
            #[cfg(windows)]
            {
                // Windows: if we can get metadata, we can usually read
                true
            }
        }
        Err(_) => false,
    }
}

pub fn is_hidden(path: &Path) -> Result<bool, std::io::Error> {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        let metadata = std::fs::metadata(path)?;
        Ok(metadata.file_attributes() & 0x2 != 0)
    }

    #[cfg(unix)]
    {
        Ok(path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.starts_with('.'))
            .unwrap_or(false))
    }
}
