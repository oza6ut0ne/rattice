use std::path::Path;

use anyhow::{anyhow, Result};

const IMAGE_EXTTENSIONS: &[&str] = &[
    "apng", "avif", "gif", "jpg", "jpeg", "jfif", "pjpeg", "pjp", "png", "svg", "webp", "bmp",
    "ico", "cur", "tif", "tiff",
];

const VIDEO_EXTTENSIONS: &[&str] = &[
    "3gp", "mpg", "mpeg", "mp4", "m4v", "m4p", "ogv", "ogg", "mov", "webm", "aac", "flac", "mp3",
    "m4a", "oga", "wav",
];

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum MediaType {
    Image,
    Video,
    Other,
}

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum File {
    Directory {
        path: String,
        name: String,
    },
    File {
        path: String,
        name: String,
        media_type: MediaType,
    },
}

impl MediaType {
    pub fn new(path: &Path) -> Self {
        match path
            .extension()
            .map(|e| e.to_string_lossy().to_string().to_ascii_lowercase())
        {
            Some(ext) => {
                if IMAGE_EXTTENSIONS.contains(&ext.as_str()) {
                    Self::Image
                } else if VIDEO_EXTTENSIONS.contains(&ext.as_str()) {
                    Self::Video
                } else {
                    Self::Other
                }
            }
            None => Self::Other,
        }
    }
}

impl File {
    pub fn new(path_buf: &Path) -> Result<File> {
        let path = path_buf
            .strip_prefix("./")?
            .to_str()
            .ok_or_else(|| anyhow!("Failed to convert path to &str: {:?}", path_buf))?
            .to_owned();

        let name = path_buf
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| path.clone());

        let file = if path_buf.is_dir() {
            Self::Directory { path, name }
        } else {
            Self::File {
                path,
                name,
                media_type: MediaType::new(path_buf),
            }
        };

        Ok(file)
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Directory { path: _, name } => name,
            Self::File {
                path: _,
                name,
                media_type: _,
            } => name,
        }
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self,
            Self::File {
                path: _,
                name: _,
                media_type: MediaType::Image,
            }
        )
    }

    pub fn is_video(&self) -> bool {
        matches!(
            self,
            Self::File {
                path: _,
                name: _,
                media_type: MediaType::Video,
            }
        )
    }
}
