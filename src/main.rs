use zip;
use encoding_rs::*;

const ENCODINGS : &'static [&'static Encoding] = &[
    UTF_8, GB18030, GBK, BIG5, EUC_JP, EUC_KR, IBM866, ISO_2022_JP, ISO_8859_10, ISO_8859_13, ISO_8859_14, ISO_8859_15,
    ISO_8859_16, ISO_8859_2, ISO_8859_3, ISO_8859_4, ISO_8859_5, ISO_8859_6, ISO_8859_7, ISO_8859_8, ISO_8859_8_I,
    KOI8_R, KOI8_U, SHIFT_JIS, UTF_16BE, UTF_16LE, MACINTOSH, REPLACEMENT, WINDOWS_1250, WINDOWS_1251, WINDOWS_1252,
    WINDOWS_1253, WINDOWS_1254, WINDOWS_1255, WINDOWS_1256, WINDOWS_1257, WINDOWS_1258, WINDOWS_874, X_MAC_CYRILLIC
];

fn try_all_decodings(bytes: &[u8]) -> (std::borrow::Cow<'_, str>, &'static Encoding, bool) {
    for &encoding in ENCODINGS {
        let (decoded, _, had_errors) = encoding.decode(bytes);
        if !had_errors {
            return (decoded, encoding, had_errors);
        }
    }
    (std::borrow::Cow::Borrowed(""), UTF_8, false)
}

fn unzip_file(zip_path: &std::path::Path, output_folder: &std::path::Path) -> Result<(), zip::result::ZipError> {
    let file = std::fs::File::open(zip_path)?;
    let mut zip_archive = zip::ZipArchive::new(file)?;
    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;
        let raw_bytes = file.name_raw().to_owned();
        let (decoded_name, _, has_error) = try_all_decodings(&raw_bytes);
        if has_error {
            eprintln!("Error decoding file name with all encodings");
            return Err(zip::result::ZipError::InvalidArchive(std::borrow::Cow::Borrowed("Failed to decode file name")));
        }
        let decoded_path = std::path::Path::new(decoded_name.as_ref());
        let output_path = output_folder.join(decoded_path);
        if file.name().ends_with('/') {
            std::fs::create_dir_all(&output_path)?;
        } else {
            if let Some(p) = output_path.parent() {
                std::fs::create_dir_all(p)?;
            }
            let mut outfile = std::fs::File::create(&output_path)?;
            std::io::copy(&mut file, &mut outfile)?;
        }
        println!("Extracted: {}", output_path.display());
    }
    Ok(())
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <zip_file_path> <output_folder>", args[0]);
        std::process::exit(1);
    }
    let zip_path = std::path::Path::new(&args[1]);
    let output_folder = if args.len() > 2 {
        std::path::Path::new(&args[2])
    } else {
        zip_path.parent().unwrap_or_else(|| {
            eprintln!("Error: No output folder specified and zip file has no parent directory.");
            std::process::exit(1);
        })
    };

    if !zip_path.exists() {
        eprintln!("Error: Zip file '{}' does not exist.", zip_path.display());
        std::process::exit(1);
    }
    if !zip_path.is_file() {
        eprintln!("Error: '{}' is not a file.", zip_path.display());
        std::process::exit(1);
    }
    unzip_file(zip_path, output_folder).unwrap_or_else(|e| {
        eprintln!("Error extracting zip file: {}", e);
        std::process::exit(1);
    });
    println!("All files extracted successfully.");
}