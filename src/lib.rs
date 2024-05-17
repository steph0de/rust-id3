#![doc = include_str!("../README.md")]
#![deny(missing_docs)]
#![deny(clippy::all)]

// Resources:
// * ID3v2.2 <http://id3.org/id3v2-00>
// * ID3v2.3 <http://id3.org/id3v2.3.0>
// * ID3v2.4 <http://id3.org/id3v2.4.0-structure>

pub use crate::error::{no_tag_ok, partial_tag_ok, Error, ErrorKind, Result};
pub use crate::frame::{Content, Frame, Timestamp};
pub use crate::storage::StorageFile;
pub use crate::stream::encoding::Encoding;
pub use crate::stream::tag::Encoder;
pub use crate::tag::{Tag, Version};
pub use crate::taglike::TagLike;

/// Contains types and methods for operating on ID3 frames.
pub mod frame;
/// Utilities for working with ID3v1 tags.
pub mod v1;
/// Combined API that handles both ID3v1 and ID3v2 tags at the same time.
pub mod v1v2;

mod chunk;
mod error;
mod storage;
mod stream;
mod tag;
mod taglike;
mod tcon;
