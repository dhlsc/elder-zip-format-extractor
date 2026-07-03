// ...existing code...
mod unziptools;
use egui::{self, ComboBox, FontData, FontDefinitions, FontFamily};
use encoding_rs::*;
use rfd::FileDialog;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use zip::ZipArchive;

struct ZipExtractor {
    zip_path: std::path::PathBuf,
    zip_archive: Option<ZipArchive<File>>,
    current_encoding: Option<&'static Encoding>,
    possible_encodings: Vec<&'static Encoding>,
    other_encodings: Vec<&'static Encoding>,
    zip_file_names: Vec<String>,
    // veriable to bind with ui
    ui_encoding: Option<&'static Encoding>,
}

impl ZipExtractor {
    fn new() -> Self {
        Self {
            zip_path: PathBuf::from("."),
            zip_archive: None,
            current_encoding: None,
            possible_encodings: Vec::new(),
            other_encodings: Vec::new(),
            zip_file_names: Vec::new(),
            ui_encoding: Some(UTF_8),
        }
    }

    fn load_archive(&mut self, path: &Path) -> bool {
        match File::open(path) {
            Ok(file) => match ZipArchive::new(file) {
                Ok(archive) => {
                    self.zip_archive = Some(archive);
                    self.zip_path = path.parent().unwrap_or(Path::new(".")).to_path_buf();

                    self.possible_encodings.clear();
                    self.other_encodings.clear();
                    self.zip_file_names.clear();
                    self.current_encoding = None;
                    self.ui_encoding = None;

                    if let Some(archive) = self.zip_archive.as_mut() {
                        let possible_encodings = if unziptools::is_all_utf8_encoded(archive) {
                            vec![UTF_8]
                        } else {
                            unziptools::get_possible_encodings(archive)
                        };
                        let (possible_encodings, other_encodings) =
                            unziptools::partition_encodings(&possible_encodings);

                        self.possible_encodings = possible_encodings;
                        self.other_encodings = other_encodings;

                        if let Some(encoding) = self.possible_encodings.first().copied() {
                            self.current_encoding = Some(encoding);
                            self.ui_encoding = self.current_encoding;
                            self.zip_file_names =
                                unziptools::get_decoded_file_names(archive, encoding);
                        }
                    }

                    true
                }
                Err(e) => {
                    eprintln!("Failed to read zip: {}", e);
                    false
                }
            },
            Err(e) => {
                eprintln!("Failed to open file: {}", e);
                false
            }
        }
    }

    fn update_preview_for_encoding(&mut self) {
        if self.ui_encoding == self.current_encoding {
            return;
        }

        self.current_encoding = self.ui_encoding;
        self.zip_file_names.clear();

        if let Some(archive) = self.zip_archive.as_mut() {
            if let Some(encoding) = self.current_encoding {
                self.zip_file_names = unziptools::get_decoded_file_names(archive, encoding);
            }
        }
    }
}

impl eframe::App for ZipExtractor {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    if let Some(path) = FileDialog::new().add_filter("zip", &["zip"]).pick_file() {
                        self.load_archive(path.as_path());
                    }
                }

                if self.zip_archive.is_some() && self.current_encoding.is_some() {
                    if ui.button("Extract").clicked() {
                        // select output folder and extract
                        if let Some(output_folder) = FileDialog::new()
                            .set_directory(self.zip_path.as_path())
                            .pick_folder()
                        {
                            if let (Some(archive), Some(encoding)) =
                                (&mut self.zip_archive, self.current_encoding)
                            {
                                match unziptools::unzip_file_with(archive, &output_folder, encoding)
                                {
                                    Ok(_) => {
                                        // show a message box using rfd
                                        rfd::MessageDialog::new()
                                            .set_title("Extraction Completed")
                                            .set_description(&format!(
                                                "Extraction completed to {}",
                                                output_folder.display()
                                            ))
                                            .set_level(rfd::MessageLevel::Info)
                                            .show();
                                    }
                                    Err(e) => {
                                        // show error message box
                                        rfd::MessageDialog::new()
                                            .set_title("Extraction Failed")
                                            .set_description(&format!("Extraction failed: {}", e))
                                            .set_level(rfd::MessageLevel::Error)
                                            .show();
                                    }
                                }
                            }
                        }
                    }
                }
            });

            ui.separator();

            if let Some(encoding) = self.current_encoding {
                ComboBox::from_label("Select Encoding")
                    .selected_text(encoding.name())
                    .show_ui(ui, |ui| {
                        ui.label("Encodings that work:");
                        for &enc in &self.possible_encodings {
                            ui.selectable_value(&mut self.ui_encoding, Some(enc), enc.name());
                        }
                        ui.label("Encodings that encountered errors:");
                        for &enc in &self.other_encodings {
                            ui.selectable_value(&mut self.ui_encoding, Some(enc), enc.name());
                        }
                    });
                self.update_preview_for_encoding();
            }

            if self.zip_archive.is_some() {
                ui.separator();
                ui.label(format!("Archive entries preview with encoding above:"));
                // rolling area for file names
                egui::ScrollArea::vertical()
                    .max_height(300.0)
                    .show(ui, |ui| {
                        for name in self.zip_file_names.iter() {
                            ui.label(name);
                        }
                    });
            }
        });
    }
}

fn load_font_candidates(fonts: &mut FontDefinitions, candidates: &[(&str, &str)]) {
    for (name, path) in candidates.iter().copied() {
        if let Ok(bytes) = std::fs::read(path) {
            fonts
                .font_data
                .insert(name.to_string(), Arc::new(FontData::from_owned(bytes)));
        }
    }

    for family in [FontFamily::Proportional, FontFamily::Monospace] {
        let fam = fonts.families.entry(family).or_default();
        for key in ["noto-emoji", "msyh", "simsun", "simhei", "arial-unicode"].iter() {
            if fonts.font_data.contains_key(&key.to_string()) && !fam.contains(&key.to_string()) {
                fam.insert(0, key.to_string());
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn load_system_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let candidates = [
        ("noto-emoji", "C:\\Windows\\Fonts\\seguiemj.ttf"),
        ("msyh", "C:\\Windows\\Fonts\\msyh.ttc"),
        ("simsun", "C:\\Windows\\Fonts\\simsun.ttc"),
        ("simhei", "C:\\Windows\\Fonts\\simhei.ttf"),
        ("arial-unicode", "C:\\Windows\\Fonts\\ARIALUNI.TTF"),
    ];
    load_font_candidates(&mut fonts, &candidates);
    fonts
}

#[cfg(target_os = "linux")]
fn load_system_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let candidates = [
        ("noto-emoji", "/usr/share/fonts/noto/NotoColorEmoji.ttf"),
        ("msyh", "/usr/share/fonts/truetype/msyh.ttc"),
        ("simsun", "/usr/share/fonts/truetype/simsun.ttc"),
        ("simhei", "/usr/share/fonts/truetype/simhei.ttf"),
        ("arial-unicode", "/usr/share/fonts/truetype/arialuni.ttf"),
    ];
    load_font_candidates(&mut fonts, &candidates);
    fonts
}

// write a gui with egui
fn main() {
    let options = eframe::NativeOptions::default();
    let _ = eframe::run_native(
        "Zip Extractor",
        options,
        Box::new(|cc| {
            cc.egui_ctx.set_fonts(load_system_fonts());

            Ok(Box::new(ZipExtractor::new()))
        }),
    );
}
