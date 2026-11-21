//! A minimal ZIP writer (method "stored", no compression, no ZIP64).
//!
//! The export files are text and small. The writer checks the names: no absolute path, no `..`.
//! The limit is 65535 entries and 4 GiB in total.

use crate::{Error, Result};

fn crc32(data: &[u8]) -> u32 {
    let mut table = [0u32; 256];
    for (i, slot) in table.iter_mut().enumerate() {
        let mut c = u32::try_from(i).unwrap_or(0);
        for _ in 0..8 {
            c = if c & 1 == 1 {
                0xEDB8_8320 ^ (c >> 1)
            } else {
                c >> 1
            };
        }
        *slot = c;
    }
    let mut crc = 0xFFFF_FFFFu32;
    for b in data {
        crc = table[usize::from(u8::try_from(crc & 0xFF).unwrap_or(0) ^ b)] ^ (crc >> 8);
    }
    !crc
}

/// Builds a ZIP archive in memory.
#[derive(Debug, Default)]
pub struct ZipWriter {
    body: Vec<u8>,
    directory: Vec<u8>,
    entries: u16,
}

fn u16le(v: &mut Vec<u8>, x: u16) {
    v.extend_from_slice(&x.to_le_bytes());
}
fn u32le(v: &mut Vec<u8>, x: u32) {
    v.extend_from_slice(&x.to_le_bytes());
}

impl ZipWriter {
    /// An empty archive.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a file.
    pub fn add(&mut self, name: &str, data: &[u8]) -> Result<()> {
        if name.is_empty()
            || name.starts_with('/')
            || name.split('/').any(|p| p == "..")
            || name.contains('\\')
            || name.len() > 255
        {
            return Err(Error::Format(format!("bad entry name {name}")));
        }
        let too_big = || Error::Format("archive is too large".into());
        let size = u32::try_from(data.len()).map_err(|_| too_big())?;
        let offset = u32::try_from(self.body.len()).map_err(|_| too_big())?;
        self.entries = self.entries.checked_add(1).ok_or_else(too_big)?;
        let crc = crc32(data);
        let name_len = u16::try_from(name.len()).map_err(|_| too_big())?;
        // Fixed time 1980-01-01 00:00: the archive is the same for the same input.
        let (time, date) = (0u16, 0x0021u16);
        let flags = 0x0800u16; // names are UTF-8
        let b = &mut self.body;
        u32le(b, 0x0403_4b50);
        u16le(b, 20);
        u16le(b, flags);
        u16le(b, 0);
        u16le(b, time);
        u16le(b, date);
        u32le(b, crc);
        u32le(b, size);
        u32le(b, size);
        u16le(b, name_len);
        u16le(b, 0);
        b.extend_from_slice(name.as_bytes());
        b.extend_from_slice(data);
        let d = &mut self.directory;
        u32le(d, 0x0201_4b50);
        u16le(d, 20);
        u16le(d, 20);
        u16le(d, flags);
        u16le(d, 0);
        u16le(d, time);
        u16le(d, date);
        u32le(d, crc);
        u32le(d, size);
        u32le(d, size);
        u16le(d, name_len);
        u16le(d, 0);
        u16le(d, 0);
        u16le(d, 0);
        u16le(d, 0);
        u32le(d, 0);
        u32le(d, offset);
        d.extend_from_slice(name.as_bytes());
        Ok(())
    }

    /// Finishes the archive.
    pub fn finish(mut self) -> Result<Vec<u8>> {
        let too_big = || Error::Format("archive is too large".into());
        let dir_offset = u32::try_from(self.body.len()).map_err(|_| too_big())?;
        let dir_size = u32::try_from(self.directory.len()).map_err(|_| too_big())?;
        let mut out = self.body;
        out.append(&mut self.directory);
        u32le(&mut out, 0x0605_4b50);
        u16le(&mut out, 0);
        u16le(&mut out, 0);
        u16le(&mut out, self.entries);
        u16le(&mut out, self.entries);
        u32le(&mut out, dir_size);
        u32le(&mut out, dir_offset);
        u16le(&mut out, 0);
        if u32::try_from(out.len()).is_err() {
            return Err(too_big());
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_matches_known_value() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
    }

    #[test]
    fn rejects_bad_names() {
        let mut z = ZipWriter::new();
        assert!(z.add("../x", b"").is_err());
        assert!(z.add("/x", b"").is_err());
        assert!(z.add("a/b.txt", b"hi").is_ok());
        let bytes = z.finish().unwrap_or_default();
        assert_eq!(&bytes[..4], b"PK\x03\x04");
    }
}
