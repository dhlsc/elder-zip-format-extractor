use encoding_rs::*;
use std::io::{Read, Seek};
use zip::{self, HasZipMetadata, read::ZipFile};

pub const ENCODINGS: &[&Encoding] = &[
    UTF_8, GB18030, GBK, BIG5, EUC_JP, EUC_KR, IBM866, ISO_2022_JP, ISO_8859_10, ISO_8859_13,
    ISO_8859_14, ISO_8859_15, ISO_8859_16, ISO_8859_2, ISO_8859_3, ISO_8859_4, ISO_8859_5,
    ISO_8859_6, ISO_8859_7, ISO_8859_8, ISO_8859_8_I, KOI8_R, KOI8_U, SHIFT_JIS, UTF_16BE,
    UTF_16LE, MACINTOSH, REPLACEMENT, WINDOWS_1250, WINDOWS_1251, WINDOWS_1252, WINDOWS_1253,
    WINDOWS_1254, WINDOWS_1255, WINDOWS_1256, WINDOWS_1257, WINDOWS_1258, WINDOWS_874,
    X_MAC_CYRILLIC,
];

#[allow(dead_code)]
pub fn is_utf8_encoded<R: Read + Seek>(zip_file: &ZipFile<'_, R>) -> bool {
    zip_file.get_metadata().is_utf8
}

#[allow(dead_code)]
pub fn get_utf8_flags<R: Read + Seek>(zip_archive: &mut zip::ZipArchive<R>) -> Vec<bool> {
    let mut flags = Vec::with_capacity(zip_archive.len());
    for i in 0..zip_archive.len() {
        let file = zip_archive.by_index(i).unwrap();
        flags.push(file.get_metadata().is_utf8);
    }
    flags
}

pub fn is_all_utf8_encoded<R: Read + Seek>(zip_archive: &mut zip::ZipArchive<R>) -> bool {
    for i in 0..zip_archive.len() {
        let file = zip_archive.by_index(i).unwrap();
        if !file.get_metadata().is_utf8 {
            return false;
        }
    }
    true
}

fn decode_archive_name(encoding: &'static Encoding, name_bytes: &[u8]) -> Option<String> {
    let (decoded, _, had_errors) = encoding.decode(name_bytes);
    if had_errors || decoded.is_empty() {
        None
    } else {
        Some(decoded.into_owned())
    }
}

pub fn get_possible_encodings<R: Read + Seek>(
    zip_archive: &mut zip::ZipArchive<R>,
) -> Vec<&'static Encoding> {
    let mut encodings = ENCODINGS.to_vec();
    for i in 0..zip_archive.len() {
        if encodings.is_empty() {
            break;
        }

        let file = zip_archive.by_index(i).unwrap();
        let name_bytes = file.name_raw();
        encodings.retain(|&encoding| decode_archive_name(encoding, name_bytes).is_some());
    }
    encodings
}

pub fn partition_encodings<'a>(
    possible_encodings: &[&'a Encoding],
) -> (Vec<&'a Encoding>, Vec<&'a Encoding>) {
    let mut other_encodings = Vec::with_capacity(ENCODINGS.len());
    for encoding in ENCODINGS.iter().copied() {
        if !possible_encodings.contains(&encoding) {
            other_encodings.push(encoding);
        }
    }
    (possible_encodings.to_vec(), other_encodings)
}

pub fn get_decoded_file_names<R: Read + Seek>(
    zip_archive: &mut zip::ZipArchive<R>,
    encoding: &'static Encoding,
) -> Vec<String> {
    let mut names = Vec::with_capacity(zip_archive.len());
    for i in 0..zip_archive.len() {
        let file = zip_archive.by_index(i).unwrap();
        if encoding == UTF_8 && file.get_metadata().is_utf8 {
            names.push(file.name().to_string());
        } else if let Some(decoded_name) = decode_archive_name(encoding, file.name_raw()) {
            names.push(decoded_name);
        } else {
            names.push(file.name().to_string());
        }
    }
    names
}

pub fn unzip_file_with<R: Read + Seek>(
    zip_archive: &mut zip::ZipArchive<R>,
    output_folder: &std::path::Path,
    encoding: &'static Encoding,
) -> Result<(), zip::result::ZipError> {
    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;
        let name_bytes = file.name_raw();
        let Some(name_decoded) = decode_archive_name(encoding, name_bytes) else {
            return Err(zip::result::ZipError::InvalidArchive(std::borrow::Cow::Borrowed(
                "Failed to decode file name",
            )));
        };

        let path_decoded = std::path::Path::new(&name_decoded);
        let output_path = output_folder.join(path_decoded);
        if name_decoded.ends_with('/') {
            std::fs::create_dir_all(&output_path)?;
        } else {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&output_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn partition_encodings_separates_selected_from_fallbacks() {
        let possible = vec![UTF_8, GB18030];
        let (selected, other) = partition_encodings(&possible);

        assert_eq!(selected, possible);
        assert!(!other.contains(&UTF_8));
        assert!(other.contains(&GBK));
        assert!(other.contains(&BIG5));
    }
}