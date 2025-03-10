use std::path::PathBuf;

use gtk4::ListBox;
use tokio::sync::mpsc::UnboundedSender;

use crate::daemon::{
    renderer::config::AppConf,
    structs::{DaemonCmdType, DaemonEvt, Dvoty},
};

use super::{
    app_launcher::underline_string,
    class::adjust_class,
    entry::{create_base_entry, DvotyUIEntry},
    DvotyContext, DvotyEntry,
};

fn get_extension_icon(ext: &str) -> Option<PathBuf> {
    let default = format!("application/x-{}", ext);
    let mime_type = match ext.to_lowercase().as_str() {
        // No extension (executable)
        "" => "application-x-executable",

        // Document formats
        "pdf" => "application-pdf",
        "txt" => "text-plain",
        "doc" => "application-msword",
        "docx" => "application-vnd.openxmlformats-officedocument.wordprocessingml.document",
        "xls" => "application-vnd.ms-excel",
        "xlsx" => "application-vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "ppt" => "application-vnd.ms-powerpoint",
        "pptx" => "application-vnd.openxmlformats-officedocument.presentationml.presentation",
        "odt" => "application-vnd.oasis.opendocument.text",
        "ods" => "application-vnd.oasis.opendocument.spreadsheet",
        "odp" => "application-vnd.oasis.opendocument.presentation",
        "rtf" => "application-rtf",
        "epub" => "application-epub+zip",
        "mobi" => "application-x-mobipocket-ebook",
        "md" | "markdown" => "text-markdown",

        // Image formats
        "png" => "image-png",
        "jpg" | "jpeg" => "image-jpeg",
        "gif" => "image-gif",
        "webp" => "image-webp",
        "tiff" | "tif" => "image-tiff",
        "svg" => "image-svg+xml",
        "bmp" => "image-bmp",
        "ico" => "image-x-icon",
        "xcf" => "image-x-xcf",

        // Video formats
        "mp4" | "avi" | "mkv" | "mov" | "webm" | "flv" | "wmv" => "media-video",

        // Audio formats
        "mp3" => "audio-mp3",
        "wav" => "audio-x-wav",
        "ogg" => "audio-ogg",
        "flac" => "audio-flac",
        "aac" => "audio-aac",
        "m4a" => "audio-mp4",

        // Archive/compression formats
        "zip" => "application-zip",
        "tar" => "application-x-tar",
        "gz" => "application-gzip",
        "bz2" => "application-x-bzip2",
        "xz" => "application-x-xz",
        "7z" => "application-x-7z-compressed",
        "rar" => "application-vnd.rar",

        // Programming languages
        "rs" => "text-x-rust",
        "py" => "text-x-python",
        "java" => "text-x-java",
        "cpp" | "cc" => "text-x-c++src",
        "c" => "text-x-csrc",
        "h" => "text-x-chdr",
        "hpp" => "text-x-c++hdr",
        "go" => "text-x-go",
        "rb" => "text-x-ruby",
        "php" => "text-x-php",
        "swift" => "text-x-swift",
        "js" => "application-javascript",
        "ts" => "application-typescript",
        "jsx" => "text-jsx",
        "tsx" => "text-tsx",

        // Configuration formats
        "json" => "application-json",
        "xml" => "application-xml",
        "toml" => "application-toml",
        "yaml" | "yml" => "application-yaml",

        // Web development
        "html" | "htm" => "text-html",
        "css" => "text-css",
        "scss" | "sass" => "text-x-scss",
        "less" => "text-x-less",
        "woff" => "font-woff",
        "woff2" => "font-woff2",
        "ttf" => "font-ttf",
        "eot" => "application-vnd.ms-fontobject",

        // System files
        "desktop" => "application-x-desktop",
        "service" => "text-plain",
        "conf" => "text-plain",
        "log" => "text-plain",
        "socket" => "application-octet-stream",
        "sh" => "application-x-shellscript",

        // Database formats
        "db" | "sqlite" => "application-vnd.sqlite3",
        "sql" => "application-sql",

        // Virtual machine/container formats
        "iso" => "application-x-iso9660-image",
        "img" => "application-x-raw-disk-image",
        "vdi" => "application-x-virtualbox-vdi",
        "vmdk" => "application-x-vmdk",
        "dockerfile" => "text-x-dockerfile",
        // Add more mappings as needed
        _ => {
            // For unknown extensions, try to construct a generic mimetype
            // or fall back to a generic file icon
            &default
        }
    };

    xdgkit::icon_finder::find_icon(mime_type.to_string(), 48, 1)
}

pub fn populate_search_entry(
    config: std::sync::Arc<AppConf>,
    list: &ListBox,
    path: PathBuf,
    name: String,
    icon: Option<PathBuf>,
    context: &mut DvotyContext,
    sender: UnboundedSender<DaemonEvt>,
    monitor: usize,
) {
    let row = create_base_entry(
        icon.unwrap_or(PathBuf::default()),
        &name,
        "Click to search",
        sender,
        config.clone(),
        monitor,
    );

    context.dvoty_entries[monitor].push((DvotyUIEntry::File { path }, row.clone()));

    if context.dvoty_entries[monitor].len() <= 1 {
        adjust_class(0, 0, &mut context.dvoty_entries[monitor]);
    }

    list.append(&row);
}

pub fn process_recent_files(
    input: String,
    sender: UnboundedSender<DaemonEvt>,
    id: &uuid::Uuid,
    monitor: usize,
    recent_paths: Vec<PathBuf>,
) {
    recent_paths
        .iter()
        .filter_map(|path| {
            if path.is_dir() {
                let name = path.file_name()?.to_str()?.trim().to_string();
                let icon = xdgkit::icon_finder::find_icon("folder".into(), 48, 1);

                return name
                    .to_lowercase()
                    .contains(&input.to_lowercase())
                    .then_some((path.to_owned(), name, icon));
            }

            let name = path.file_name()?.to_str()?.trim().to_string();

            let extension = path
                .extension()
                .map_or("", |v| v.to_str().map_or("", |v| v))
                .trim();

            let icon = get_extension_icon(extension);

            name.to_lowercase()
                .contains(&input.to_lowercase())
                .then_some((path.to_owned(), name, icon))
        })
        .for_each(|(path, name, icon)| {
            let str = path.to_str().unwrap_or("").to_string();
            sender
                .send(DaemonEvt {
                    evt: DaemonCmdType::Dvoty(Dvoty::AddEntry(DvotyEntry::File {
                        path,
                        name: format!(
                            "{} <span color=\"grey\"><i>{}</i></span>",
                            underline_string(&input, &name),
                            str
                        ),
                        icon,
                    })),
                    sender: None,
                    uuid: Some(*id),
                    monitors: vec![monitor],
                })
                .unwrap_or_else(|e| {
                    println!("Dvoty: Cannot send file: {}", e);
                })
        });
}
