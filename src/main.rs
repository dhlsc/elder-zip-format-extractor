use zip;
use encoding_rs::*;

const ENCODINGS : &'static [&'static Encoding] = &[
    UTF_8, GB18030, GBK, BIG5, EUC_JP, EUC_KR, IBM866, ISO_2022_JP, ISO_8859_10, ISO_8859_13, ISO_8859_14, ISO_8859_15,
    ISO_8859_16, ISO_8859_2, ISO_8859_3, ISO_8859_4, ISO_8859_5, ISO_8859_6, ISO_8859_7, ISO_8859_8, ISO_8859_8_I,
    KOI8_R, KOI8_U, SHIFT_JIS, UTF_16BE, UTF_16LE, MACINTOSH, REPLACEMENT, WINDOWS_1250, WINDOWS_1251, WINDOWS_1252,
    WINDOWS_1253, WINDOWS_1254, WINDOWS_1255, WINDOWS_1256, WINDOWS_1257, WINDOWS_1258, WINDOWS_874, X_MAC_CYRILLIC
];

fn get_possible_encodings(zip_path: &std::path::Path) -> Vec<&'static Encoding>{
    let mut encodings = ENCODINGS.to_vec();
    let file = std::fs::File::open(zip_path).unwrap_or_else(|e| {
        eprintln!("Error: Could not open zip file: {}", e);
        std::process::exit(1);
    });
    let mut zip_archive = zip::ZipArchive::new(file).unwrap_or_else(|e| {
        eprintln!("Error: Could not read zip archive: {}", e);
        std::process::exit(1);
    });
    for i in 0..zip_archive.len() {
        let file = zip_archive.by_index(i).unwrap_or_else(|e| {
            eprintln!("Error: Could not read file at index {}: {}", i, e);
            std::process::exit(1);
        });
        let raw_bytes = file.name_raw().to_owned();
        encodings.retain(|&encoding| {
            let (decoded, _, had_errors) = encoding.decode(&raw_bytes);
            !had_errors && !decoded.is_empty()
        });
    }
    encodings
}

fn print_file_path_with_decoding(zip_path: &std::path::Path, encoding: &'static Encoding) {
    let file = std::fs::File::open(zip_path).unwrap_or_else(|e| {
        eprintln!("Error: Could not open zip file: {}", e);
        std::process::exit(1);
    });
    let mut zip_archive = zip::ZipArchive::new(file).unwrap_or_else(|e| {
        eprintln!("Error: Could not read zip archive: {}", e);
        std::process::exit(1);
    });
    println!("List file path with encoding : {}", encoding.name());
    for i in 0..zip_archive.len() {
        let file = zip_archive.by_index(i).unwrap_or_else(|e| {
            eprintln!("Error: Could not read file at index {}: {}", i, e);
            std::process::exit(1);
        });
        let raw_bytes = file.name_raw().to_owned();
        let (decoded_name, _, has_error) = encoding.decode(&raw_bytes);
        if has_error {
            eprintln!("Error : decoding file name error!");
            continue;
        }
        println!("File : {}", decoded_name);
    }
}

fn unzip_file_with(zip_path: &std::path::Path, output_folder: &std::path::Path, encoding: &'static Encoding) -> Result<(), zip::result::ZipError> {
    let file = std::fs::File::open(zip_path)?;
    let mut zip_archive = zip::ZipArchive::new(file)?;
    for i in 0..zip_archive.len() {
        let mut file = zip_archive.by_index(i)?;
        let raw_bytes = file.name_raw().to_owned();
        let (decoded_name, _, has_error) = encoding.decode(&raw_bytes);
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

const USAGE: &str = r#"usage:
extract : {0} x <zip_file_path> [output_folder] [encoding]
    Extracts files from a zip archive with specified encoding.
    If no encoding is specified, it will try to decode with the first possible encodings.
        If no possible encodings, an error is print.
    If output_folder is not specified, it will use the parent directory of the zip file.
test : {0} t <zip_file_path>
    Tests all possible encodings of the zip archive and print it.
show : {0} s <zip_file_path> [encoding]
    Show the possible encodings of zip archive and lists file names decoded with them.
    If encoding is specified, it will only list file path decoded with specify encoding.
"#;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        eprintln!("{}", USAGE.replace("{0}", &args[0]));
        std::process::exit(1);
    }
    let option = args[1].as_str();
    match option {
        "x" => {
            let zip_path = std::path::Path::new(&args[2]);
            let output_folder = if args.len() > 3 {
                std::path::Path::new(&args[3])
            } else {
                zip_path.parent().unwrap_or_else(|| {
                    eprintln!("Error: No output folder specified and zip file has no parent directory.");
                    std::process::exit(1);
                })
            };
            let encoding = if args.len() > 4 {
                let enc_name = &args[4];
                ENCODINGS.iter().find(|e| e.name() == enc_name.as_str()).copied().unwrap_or_else(|| {
                    eprintln!("Error: Encoding '{}' not found. Using UTF-8 as default.", enc_name);
                    std::process::exit(1);
                })
            } else {
                *get_possible_encodings(zip_path).first().unwrap_or_else(|| {
                    eprintln!("Error: No possible encodings found for the zip file.");
                    std::process::exit(1);
                })
            };
            unzip_file_with(zip_path, output_folder, encoding).unwrap_or_else(|e| {
                eprintln!("Error extracting zip file: {}", e);
                std::process::exit(1);
            });
            println!("All files extracted successfully.");
            std::process::exit(0);
        }
        "t" => {
            let zip_path = std::path::Path::new(&args[2]);
            let possible_encodings = get_possible_encodings(zip_path);
            println!("Possible encodings:");
            println!("{}", possible_encodings.iter().map(|e| e.name()).collect::<Vec<&str>>().join(", "));
        }
        "s" => {
            let zip_path = std::path::Path::new(&args[2]);
            if args.len() > 3 {
                let enc_name = &args[3];
                let encoding = ENCODINGS.iter().find(|e| e.name() == enc_name.as_str()).copied().unwrap_or_else(|| {
                    eprintln!("Error: Encoding '{}' not found.", enc_name);
                    std::process::exit(1);
                });
                print_file_path_with_decoding(zip_path, encoding);
            } else {
                let possible_encodings = get_possible_encodings(zip_path);
                println!("Possible encodings:");
                println!("{}", possible_encodings.iter().map(|e| e.name()).collect::<Vec<&str>>().join(", "));
                for enc in possible_encodings {
                    print_file_path_with_decoding(zip_path, enc);
                }
            }
        }
        _ => {
            eprintln!("{}", USAGE.replace("{0}", &args[0]));
            std::process::exit(1);
        }
    }
}