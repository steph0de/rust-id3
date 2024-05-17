use crate::{v1, Error, ErrorKind, StorageFile, Tag, Version};
use std::fs;
use std::fs::File;
use std::io;
use std::path::Path;

/// Returns which tags are present in the specified file.
pub fn is_candidate(mut file: impl io::Read + io::Seek) -> crate::Result<FormatVersion> {
    let v2 = Tag::is_candidate(&mut file)?;
    let v1 = v1::Tag::is_candidate(&mut file)?;
    Ok(match (v1, v2) {
        (false, false) => FormatVersion::None,
        (true, false) => FormatVersion::Id3v1,
        (false, true) => FormatVersion::Id3v2,
        (true, true) => FormatVersion::Both,
    })
}

/// Returns which tags are present in the specified file.
pub fn is_candidate_path(path: impl AsRef<Path>) -> crate::Result<FormatVersion> {
    is_candidate(File::open(path)?)
}

/// Attempts to read an ID3v2 or ID3v1 tag, in that order.
///
/// If neither version tag is found, an error with [`ErrorKind::NoTag`] is returned.
pub fn read_from(mut file: impl io::Read + io::Seek) -> crate::Result<Tag> {
    match Tag::read_from2(&mut file) {
        Err(Error {
            kind: ErrorKind::NoTag,
            ..
        }) => {}
        Err(err) => return Err(err),
        Ok(tag) => return Ok(tag),
    }

    match v1::Tag::read_from(file) {
        Err(Error {
            kind: ErrorKind::NoTag,
            ..
        }) => {}
        Err(err) => return Err(err),
        Ok(tag) => return Ok(tag.into()),
    }

    Err(Error::new(
        ErrorKind::NoTag,
        "Neither a ID3v2 or ID3v1 tag was found",
    ))
}

/// Attempts to read an ID3v2 or ID3v1 tag, in that order.
///
/// If neither version tag is found, an error with [`ErrorKind::NoTag`] is returned.
pub fn read_from_path(path: impl AsRef<Path>) -> crate::Result<Tag> {
    read_from(File::open(path)?)
}

/// Writes the specified tag to a file. Any existing ID3v2 tag is replaced or added if it is not
/// present.
///
/// If any ID3v1 tag is present it will be REMOVED as it is not able to fully represent a ID3v2
/// tag.
pub fn write_to_file(mut file: impl StorageFile, tag: &Tag, version: Version) -> crate::Result<()> {
    tag.write_to_file(&mut file, version)?;
    v1::Tag::remove_from_file(&mut file)?;
    Ok(())
}

/// Conventience function for [`write_to_file`].
pub fn write_to_path(path: impl AsRef<Path>, tag: &Tag, version: Version) -> crate::Result<()> {
    let file = fs::OpenOptions::new().read(true).write(true).open(path)?;
    write_to_file(file, tag, version)
}

/// Ensures that both ID3v1 and ID3v2 are not present in the specified file.
///
/// Returns [`FormatVersion`] representing the previous state.
pub fn remove_from_path(path: impl AsRef<Path>) -> crate::Result<FormatVersion> {
    let v2 = Tag::remove_from_path(&path)?;
    let v1 = v1::Tag::remove_from_path(path)?;
    Ok(match (v1, v2) {
        (false, false) => FormatVersion::None,
        (true, false) => FormatVersion::Id3v1,
        (false, true) => FormatVersion::Id3v2,
        (true, true) => FormatVersion::Both,
    })
}

/// An enum that represents the precense state of both tag format versions.
#[derive(Copy, Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq)]
pub enum FormatVersion {
    /// No tags.
    None,
    /// ID3v1
    Id3v1,
    /// ID3v2
    Id3v2,
    /// ID3v1 + ID3v2
    Both,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::TagLike;
    use std::fs::File;
    use std::io::{copy, Write};

    fn file_with_both_formats() -> tempfile::NamedTempFile {
        // Write both ID3v1 and ID3v2 tags to a single file, the ID3v2 should be prefered when
        // reading.
        let mut v2_testdata = File::open("testdata/id3v24.id3").unwrap();
        let mut v1_testdata = File::open("testdata/id3v1.id3").unwrap();
        let mut tmp = tempfile::NamedTempFile::new().unwrap();
        copy(&mut v2_testdata, &mut tmp).unwrap();
        tmp.write_all(&[0xaa; 1337]).unwrap(); // Dummy data, can be anything.
        copy(&mut v1_testdata, &mut tmp).unwrap();
        tmp
    }

    #[test]
    fn test_is_candidate() {
        let tmp = file_with_both_formats();
        assert_eq!(is_candidate_path(&tmp).unwrap(), FormatVersion::Both);
        assert_eq!(
            is_candidate_path("testdata/image.jpg").unwrap(),
            FormatVersion::None
        );
        assert_eq!(
            is_candidate_path("testdata/id3v1.id3").unwrap(),
            FormatVersion::Id3v1
        );
        assert_eq!(
            is_candidate_path("testdata/id3v24.id3").unwrap(),
            FormatVersion::Id3v2
        );
    }

    #[test]
    fn test_read_from_path() {
        let tmp = file_with_both_formats();

        let v2 = read_from_path(&tmp).unwrap();
        assert_eq!(v2.genre(), Some("Genre"));

        let v1 = read_from_path("testdata/id3v1.id3").unwrap();
        assert_eq!(v1.genre(), Some("Trance"));
    }

    #[test]
    fn test_write_to_path() {
        let tmp = file_with_both_formats();

        let mut tag = read_from_path(&tmp).unwrap();
        tag.set_artist("High Contrast");
        write_to_path(&tmp, &tag, Version::Id3v24).unwrap();

        assert_eq!(is_candidate_path(&tmp).unwrap(), FormatVersion::Id3v2);
    }

    #[test]
    fn test_remove_from_path() {
        let tmp = file_with_both_formats();

        assert_eq!(remove_from_path(&tmp).unwrap(), FormatVersion::Both);
    }
}
