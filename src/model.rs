use std::{cmp::Ordering, fs::Metadata, path::Path, str::FromStr, time::SystemTime};

use anyhow::{anyhow, Result};
use percent_encoding::{utf8_percent_encode, AsciiSet, NON_ALPHANUMERIC};

const FRAGMENT: &AsciiSet = &NON_ALPHANUMERIC.remove(b'/').remove(b'.');

const IMAGE_EXTENSIONS: &[&str] = &[
    "apng", "avif", "gif", "jpg", "jpeg", "jfif", "pjpeg", "pjp", "png", "svg", "webp", "bmp",
    "ico", "cur", "tif", "tiff",
];

const VIDEO_EXTENSIONS: &[&str] = &[
    "3gp", "mpg", "mpeg", "mp4", "m4v", "m4p", "ogv", "ogg", "mov", "webm", "aac", "flac", "mp3",
    "m4a", "oga", "wav",
];

#[derive(Clone)]
pub enum SortOrder {
    Name,
    CreatedAt,
    ModifiedAt,
}

impl FromStr for SortOrder {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "name" => Ok(Self::Name),
            "created" => Ok(Self::CreatedAt),
            "modified" => Ok(Self::ModifiedAt),
            _ => Err(format!("Invalid variant name: {}", s)),
        }
    }
}

pub(crate) enum MediaType {
    Image,
    Video,
    Other,
}

pub(crate) enum File {
    Directory {
        name: String,
        path: String,
        metadata: Option<Metadata>,
    },
    File {
        name: String,
        path: String,
        media_type: MediaType,
        metadata: Option<Metadata>,
    },
}

impl MediaType {
    pub fn new(path: &Path) -> Self {
        match path
            .extension()
            .map(|e| e.to_string_lossy().to_ascii_lowercase())
        {
            Some(ext) => {
                if IMAGE_EXTENSIONS.contains(&ext.as_str()) {
                    Self::Image
                } else if VIDEO_EXTENSIONS.contains(&ext.as_str()) {
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
    pub fn new(path_ref: &Path, metadata: Option<Metadata>) -> Result<Self> {
        let name = path_ref
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .map_or_else(|| Self::path_string_from_path_ref(path_ref), Ok)?;

        Self::new_with_name(path_ref, name, metadata)
    }

    pub fn new_with_name<T>(path_ref: &Path, name: T, metadata: Option<Metadata>) -> Result<Self>
    where
        T: Into<String>,
    {
        let mut path = Self::path_string_from_path_ref(path_ref)?;
        let name = name.into();

        let file = if path_ref.is_dir() {
            if !path.is_empty() {
                path.push('/')
            }
            Self::Directory {
                name,
                path,
                metadata,
            }
        } else {
            Self::File {
                name,
                path,
                media_type: MediaType::new(path_ref),
                metadata,
            }
        };

        Ok(file)
    }

    fn path_string_from_path_ref(path: &Path) -> Result<String> {
        path.strip_prefix("./")?
            .to_str()
            .map(|p| utf8_percent_encode(p, FRAGMENT).to_string())
            .ok_or_else(|| anyhow!("Failed to convert path to &str: {:?}", path))
    }

    pub fn cmp_by(&self, other: &Self, order: &SortOrder, reverse: bool) -> Ordering {
        match (self.is_dir(), other.is_dir()) {
            (true, false) => return Ordering::Less,
            (false, true) => return Ordering::Greater,
            _ => { /* nop. */ }
        };

        let ordering = match order {
            SortOrder::Name => self.name().cmp(other.name()),
            SortOrder::CreatedAt => self.cmp_by_created_at(other),
            SortOrder::ModifiedAt => self.cmp_by_modified_at(other),
        };

        if reverse {
            ordering.reverse()
        } else {
            ordering
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Directory {
                name,
                path: _,
                metadata: _,
            } => name,
            Self::File {
                name,
                path: _,
                media_type: _,
                metadata: _,
            } => name,
        }
    }

    fn is_dir(&self) -> bool {
        matches!(
            self,
            Self::Directory {
                name: _,
                path: _,
                metadata: _,
            }
        )
    }

    pub fn is_image(&self) -> bool {
        matches!(
            self,
            Self::File {
                name: _,
                path: _,
                media_type: MediaType::Image,
                metadata: _,
            }
        )
    }

    pub fn is_video(&self) -> bool {
        matches!(
            self,
            Self::File {
                name: _,
                path: _,
                media_type: MediaType::Video,
                metadata: _,
            }
        )
    }

    fn metadata(&self) -> &Option<Metadata> {
        match self {
            Self::Directory {
                name: _,
                path: _,
                metadata,
            } => metadata,
            Self::File {
                name: _,
                path: _,
                media_type: _,
                metadata,
            } => metadata,
        }
    }

    fn created_at(&self) -> Option<SystemTime> {
        self.metadata().clone().and_then(|m| m.created().ok())
    }

    fn modified_at(&self) -> Option<SystemTime> {
        self.metadata().clone().and_then(|m| m.modified().ok())
    }

    fn cmp_by_created_at(&self, other: &Self) -> Ordering {
        match (self.created_at(), other.created_at()) {
            (Some(s), Some(o)) => s.cmp(&o),
            _ => Ordering::Equal,
        }
    }

    fn cmp_by_modified_at(&self, other: &Self) -> Ordering {
        match (self.modified_at(), other.modified_at()) {
            (Some(s), Some(o)) => s.cmp(&o),
            _ => Ordering::Equal,
        }
    }
}
