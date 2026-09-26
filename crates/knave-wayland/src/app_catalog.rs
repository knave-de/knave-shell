use std::{
    collections::HashSet,
    fs::{self, File},
    io::{Cursor, Read},
    path::{Path, PathBuf},
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

use freedesktop_desktop_entry::DesktopEntry;
use knave_ui::{ApplicationSummary, UiImage};

const MAX_APPLICATIONS: usize = 512;
const MAX_DESKTOP_FILES: usize = 2_048;
const MAX_DESKTOP_DIRECTORIES: usize = 1_024;
const MAX_DIRECTORY_ENTRIES: usize = 8_192;
const MAX_DESKTOP_DEPTH: usize = 8;
const MAX_DESKTOP_ENTRY_BYTES: u64 = 64 * 1024;
const MAX_DESKTOP_TOTAL_BYTES: u64 = 8 * 1024 * 1024;
const MAX_ICON_FILE_BYTES: u64 = 1024 * 1024;
const MAX_ICON_DIMENSION: u32 = 512;
const ICON_SIZE: u32 = 48;
const MAX_ICON_CACHE: usize = 48;

pub(crate) fn load_brand_mark() -> Option<UiImage> {
    decode_svg_icon(include_bytes!(
        "../../../assets/branding/Knave-Monochrome-Dark.svg"
    ))
}

#[derive(Debug, Default)]
pub(crate) struct AppCatalogUpdate {
    pub applications: Option<Vec<ApplicationSummary>>,
    pub icons: Vec<(String, UiImage)>,
}

#[derive(Default)]
struct Control {
    pending_icons: Option<Vec<String>>,
    stop: bool,
}

pub(crate) struct AppCatalogWorker {
    control: Arc<(Mutex<Control>, Condvar)>,
    updates: Arc<Mutex<Option<AppCatalogUpdate>>>,
    thread: Option<JoinHandle<()>>,
}

impl AppCatalogWorker {
    pub(crate) fn start(wake: crate::WakeSender) -> Self {
        let control = Arc::new((Mutex::new(Control::default()), Condvar::new()));
        let updates = Arc::new(Mutex::new(None));
        let worker_control = Arc::clone(&control);
        let worker_updates = Arc::clone(&updates);
        let thread = thread::spawn(move || {
            let entries = scan_applications();
            publish_update(
                &worker_updates,
                &wake,
                AppCatalogUpdate {
                    applications: Some(entries.iter().map(|entry| entry.summary.clone()).collect()),
                    icons: Vec::new(),
                },
            );

            let mut icon_cache: Vec<(String, UiImage)> = Vec::new();
            loop {
                let request = {
                    let (lock, ready) = &*worker_control;
                    let Ok(control) = lock.lock() else {
                        break;
                    };
                    let mut control = match ready.wait_while(control, |state| {
                        state.pending_icons.is_none() && !state.stop
                    }) {
                        Ok(control) => control,
                        Err(_) => break,
                    };
                    if control.stop {
                        break;
                    }
                    control.pending_icons.take().unwrap_or_default()
                };

                let mut icons = Vec::new();
                let mut seen = HashSet::new();
                for requested_id in request {
                    let normalized = normalize_app_id(&requested_id);
                    let Some(entry) = entries.iter().find(|entry| {
                        entry
                            .summary
                            .window_app_ids
                            .iter()
                            .any(|alias| normalize_app_id(alias) == normalized)
                    }) else {
                        continue;
                    };
                    if !seen.insert(entry.summary.id.clone()) {
                        continue;
                    }

                    let icon = icon_cache
                        .iter()
                        .find(|(id, _)| id == &entry.summary.id)
                        .map(|(_, icon)| icon.clone())
                        .or_else(|| entry.icon_name.as_deref().and_then(load_icon));
                    let Some(icon) = icon else {
                        continue;
                    };

                    if !icon_cache.iter().any(|(id, _)| id == &entry.summary.id) {
                        if icon_cache.len() >= MAX_ICON_CACHE {
                            icon_cache.remove(0);
                        }
                        icon_cache.push((entry.summary.id.clone(), icon.clone()));
                    }
                    icons.push((entry.summary.id.clone(), icon));
                    if icons.len() >= MAX_ICON_CACHE {
                        break;
                    }
                }

                if !icons.is_empty() {
                    publish_update(
                        &worker_updates,
                        &wake,
                        AppCatalogUpdate {
                            applications: None,
                            icons,
                        },
                    );
                }
            }
        });

        Self {
            control,
            updates,
            thread: Some(thread),
        }
    }

    pub(crate) fn request_icons(&self, ids: Vec<String>) {
        let mut unique: Vec<String> = Vec::with_capacity(ids.len().min(MAX_ICON_CACHE));
        for id in ids {
            if unique.len() >= MAX_ICON_CACHE {
                break;
            }
            if !unique
                .iter()
                .any(|existing| normalize_app_id(existing) == normalize_app_id(&id))
            {
                unique.push(id);
            }
        }
        let (lock, ready) = &*self.control;
        if let Ok(mut control) = lock.lock() {
            control.pending_icons = Some(unique);
            ready.notify_one();
        }
    }

    pub(crate) fn latest(&self) -> Option<AppCatalogUpdate> {
        self.updates.lock().ok()?.take()
    }
}

impl Drop for AppCatalogWorker {
    fn drop(&mut self) {
        let (lock, ready) = &*self.control;
        if let Ok(mut control) = lock.lock() {
            control.stop = true;
            ready.notify_one();
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

fn publish_update(
    updates: &Mutex<Option<AppCatalogUpdate>>,
    wake: &crate::WakeSender,
    mut update: AppCatalogUpdate,
) {
    if let Ok(mut latest) = updates.lock() {
        if let Some(previous) = latest.take() {
            update.applications = update.applications.or(previous.applications);
            update.icons.splice(0..0, previous.icons);
            if update.icons.len() > MAX_ICON_CACHE {
                update.icons.drain(..update.icons.len() - MAX_ICON_CACHE);
            }
        }
        *latest = Some(update);
    }
    let _ = wake.try_send(crate::RuntimeWake::Redraw);
}

struct CatalogEntry {
    summary: ApplicationSummary,
    icon_name: Option<String>,
}

#[derive(Default)]
struct ScanBounds {
    files: usize,
    directories: usize,
    directory_entries: usize,
    total_bytes: u64,
    exhausted: bool,
}

fn scan_applications() -> Vec<CatalogEntry> {
    let mut bounds = ScanBounds::default();
    let mut entries = Vec::new();
    let mut seen = HashSet::new();
    let locales: [&str; 0] = [];

    for root in freedesktop_desktop_entry::default_paths() {
        scan_directory(&root, 0, &locales, &mut bounds, &mut seen, &mut entries);
        if bounds.exhausted || entries.len() >= MAX_APPLICATIONS {
            break;
        }
    }

    entries.sort_by(|left, right| {
        left.summary
            .name
            .to_lowercase()
            .cmp(&right.summary.name.to_lowercase())
            .then_with(|| left.summary.id.cmp(&right.summary.id))
    });
    entries
}

fn scan_directory(
    directory: &Path,
    depth: usize,
    locales: &[&str],
    bounds: &mut ScanBounds,
    seen: &mut HashSet<String>,
    entries: &mut Vec<CatalogEntry>,
) {
    if depth > MAX_DESKTOP_DEPTH
        || bounds.exhausted
        || bounds.directories >= MAX_DESKTOP_DIRECTORIES
        || entries.len() >= MAX_APPLICATIONS
    {
        return;
    }
    bounds.directories += 1;

    let Ok(read_dir) = fs::read_dir(directory) else {
        return;
    };
    let mut paths = Vec::new();
    for entry in read_dir {
        if bounds.directory_entries >= MAX_DIRECTORY_ENTRIES {
            bounds.exhausted = true;
            return;
        }
        bounds.directory_entries += 1;
        if let Ok(entry) = entry {
            paths.push(entry.path());
        }
    }
    paths.sort();

    for path in paths {
        if bounds.exhausted || entries.len() >= MAX_APPLICATIONS {
            return;
        }
        if path.is_dir() {
            scan_directory(&path, depth + 1, locales, bounds, seen, entries);
            continue;
        }
        if path.extension().and_then(|extension| extension.to_str()) != Some("desktop") {
            continue;
        }
        if bounds.files >= MAX_DESKTOP_FILES {
            bounds.exhausted = true;
            return;
        }
        bounds.files += 1;

        let Ok(metadata) = fs::metadata(&path) else {
            continue;
        };
        let length = metadata.len();
        if length == 0 || length > MAX_DESKTOP_ENTRY_BYTES {
            continue;
        }
        if bounds.total_bytes.saturating_add(length) > MAX_DESKTOP_TOTAL_BYTES {
            bounds.exhausted = true;
            return;
        }
        let Ok(mut file) = File::open(&path) else {
            continue;
        };
        let mut bytes = Vec::with_capacity(length as usize);
        if file
            .by_ref()
            .take(MAX_DESKTOP_ENTRY_BYTES + 1)
            .read_to_end(&mut bytes)
            .is_err()
            || bytes.len() as u64 > MAX_DESKTOP_ENTRY_BYTES
        {
            continue;
        }
        bounds.total_bytes = bounds.total_bytes.saturating_add(bytes.len() as u64);
        let Ok(source) = std::str::from_utf8(&bytes) else {
            continue;
        };
        let Ok(entry) = DesktopEntry::from_str(&path, source, Some(locales)) else {
            continue;
        };
        if entry.hidden()
            || entry.no_display()
            || entry.type_().is_some_and(|kind| kind != "Application")
            || entry.exec().is_none()
        {
            continue;
        }
        let Ok(argv) = entry.parse_exec() else {
            continue;
        };
        if argv.is_empty() || !seen.insert(entry.appid.clone()) {
            continue;
        }

        let name = entry
            .name(locales)
            .map(|value| value.into_owned())
            .unwrap_or_else(|| entry.appid.clone());
        let generic_name = entry
            .generic_name(locales)
            .map(|value| value.into_owned())
            .unwrap_or_default();
        let keywords = entry
            .keywords(locales)
            .unwrap_or_default()
            .into_iter()
            .map(|value| value.into_owned())
            .take(32)
            .collect();
        let mut window_app_ids = vec![entry.appid.clone()];
        if let Some(app_id) = entry.appid.strip_suffix(".desktop") {
            window_app_ids.push(app_id.to_owned());
        }
        if let Some(class) = entry.startup_wm_class()
            && !class.is_empty()
        {
            window_app_ids.push(class.to_owned());
        }
        let mut aliases = HashSet::new();
        window_app_ids.retain(|alias| aliases.insert(normalize_app_id(alias)));

        entries.push(CatalogEntry {
            summary: ApplicationSummary {
                id: entry.appid.clone(),
                name,
                generic_name,
                keywords,
                window_app_ids,
                exec_argv: argv,
                icon: None,
            },
            icon_name: entry.icon().map(str::to_owned),
        });
    }
}

fn normalize_app_id(value: &str) -> String {
    value.trim().trim_end_matches(".desktop").to_lowercase()
}

fn load_icon(icon_name: &str) -> Option<UiImage> {
    let path = if Path::new(icon_name).is_absolute() {
        PathBuf::from(icon_name)
    } else {
        freedesktop_icons::lookup(icon_name)
            .with_size(ICON_SIZE as u16)
            .find()?
    };
    let metadata = fs::metadata(&path).ok()?;
    if metadata.len() == 0 || metadata.len() > MAX_ICON_FILE_BYTES {
        return None;
    }
    let mut file = File::open(&path).ok()?;
    let mut bytes = Vec::with_capacity(metadata.len() as usize);
    file.by_ref()
        .take(MAX_ICON_FILE_BYTES + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    if bytes.len() as u64 > MAX_ICON_FILE_BYTES {
        return None;
    }

    match path.extension().and_then(|extension| extension.to_str()) {
        Some("svg") => decode_svg_icon(&bytes),
        Some("png") => decode_png_icon(&bytes),
        _ => None,
    }
}

fn decode_svg_icon(bytes: &[u8]) -> Option<UiImage> {
    let options = resvg::usvg::Options::default();
    let tree = resvg::usvg::Tree::from_data_nested(bytes, &options).ok()?;
    let size = tree.size();
    let width = size.width();
    let height = size.height();
    if !width.is_finite()
        || !height.is_finite()
        || width <= 0.0
        || height <= 0.0
        || width > 8192.0
        || height > 8192.0
    {
        return None;
    }

    let scale = (ICON_SIZE as f32 / width).min(ICON_SIZE as f32 / height);
    let offset_x = (ICON_SIZE as f32 - width * scale) / 2.0;
    let offset_y = (ICON_SIZE as f32 - height * scale) / 2.0;
    let transform =
        resvg::tiny_skia::Transform::from_scale(scale, scale).post_translate(offset_x, offset_y);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(ICON_SIZE, ICON_SIZE)?;
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    UiImage::from_rgba(ICON_SIZE, ICON_SIZE, pixmap.take())
}

fn decode_png_icon(bytes: &[u8]) -> Option<UiImage> {
    let mut decoder = png::Decoder::new(Cursor::new(bytes));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().ok()?;
    let info = reader.info();
    if info.width == 0
        || info.height == 0
        || info.width > MAX_ICON_DIMENSION
        || info.height > MAX_ICON_DIMENSION
    {
        return None;
    }
    let pixel_count = u64::from(info.width).checked_mul(u64::from(info.height))?;
    let max_bytes = usize::try_from(pixel_count.checked_mul(4)?).ok()?;
    let output_size = reader.output_buffer_size()?;
    if output_size > max_bytes {
        return None;
    }
    let mut buffer = vec![0; output_size];
    let frame = reader.next_frame(&mut buffer).ok()?;
    let data = &buffer[..frame.buffer_size()];
    let rgba = match frame.color_type {
        png::ColorType::Rgba => data.to_vec(),
        png::ColorType::Rgb => data
            .chunks(3)
            .filter(|pixel| pixel.len() == 3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),
        png::ColorType::Grayscale => data
            .iter()
            .flat_map(|value| [*value, *value, *value, 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => data
            .chunks(2)
            .filter(|pixel| pixel.len() == 2)
            .flat_map(|pixel| [pixel[0], pixel[0], pixel[0], pixel[1]])
            .collect(),
        png::ColorType::Indexed => return None,
    };
    fit_rgba_icon(frame.width, frame.height, &rgba)
}

fn fit_rgba_icon(width: u32, height: u32, pixels: &[u8]) -> Option<UiImage> {
    let expected = usize::try_from(
        u64::from(width)
            .checked_mul(u64::from(height))?
            .checked_mul(4)?,
    )
    .ok()?;
    if width == 0 || height == 0 || pixels.len() != expected {
        return None;
    }

    let scale = (ICON_SIZE as f32 / width as f32).min(ICON_SIZE as f32 / height as f32);
    let target_width = (width as f32 * scale).round().max(1.0) as u32;
    let target_height = (height as f32 * scale).round().max(1.0) as u32;
    let offset_x = (ICON_SIZE - target_width) / 2;
    let offset_y = (ICON_SIZE - target_height) / 2;
    let mut output = vec![0; (ICON_SIZE * ICON_SIZE * 4) as usize];

    for y in 0..target_height {
        let source_y = ((y as f32 + 0.5) / scale - 0.5).clamp(0.0, height as f32 - 1.0);
        let y0 = source_y.floor() as u32;
        let y1 = (y0 + 1).min(height - 1);
        let fy = source_y - y0 as f32;
        for x in 0..target_width {
            let source_x = ((x as f32 + 0.5) / scale - 0.5).clamp(0.0, width as f32 - 1.0);
            let x0 = source_x.floor() as u32;
            let x1 = (x0 + 1).min(width - 1);
            let fx = source_x - x0 as f32;
            let samples = [
                (x0, y0, (1.0 - fx) * (1.0 - fy)),
                (x1, y0, fx * (1.0 - fy)),
                (x0, y1, (1.0 - fx) * fy),
                (x1, y1, fx * fy),
            ];
            let mut alpha = 0.0;
            let mut color = [0.0; 3];
            for (sample_x, sample_y, weight) in samples {
                let source = ((sample_y * width + sample_x) * 4) as usize;
                let sample_alpha = pixels[source + 3] as f32 / 255.0;
                alpha += sample_alpha * weight;
                for channel in 0..3 {
                    color[channel] +=
                        pixels[source + channel] as f32 / 255.0 * sample_alpha * weight;
                }
            }
            let target = (((offset_y + y) * ICON_SIZE + offset_x + x) * 4) as usize;
            if alpha > 0.0 {
                for channel in 0..3 {
                    output[target + channel] = ((color[channel] / alpha) * 255.0).round() as u8;
                }
            }
            output[target + 3] = (alpha * 255.0).round() as u8;
        }
    }
    UiImage::from_rgba(ICON_SIZE, ICON_SIZE, output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_ids_match_desktop_entry_names_without_suffix() {
        assert_eq!(
            normalize_app_id("org.example.Editor.desktop"),
            "org.example.editor"
        );
    }

    #[test]
    fn icon_fit_smoothly_resamples_edges() {
        let icon = fit_rgba_icon(2, 1, &[0, 0, 0, 255, 255, 255, 255, 255]).unwrap();
        let row = &icon.pixels()[12 * 48 * 4..13 * 48 * 4];
        assert!(
            row.chunks(4)
                .any(|pixel| pixel.len() == 4 && pixel[0] > 0 && pixel[0] < 255)
        );
    }

    #[test]
    fn icon_fit_preserves_aspect_and_bounds_pixels() {
        let pixels = vec![255; 96 * 48 * 4];
        let icon = fit_rgba_icon(96, 48, &pixels).unwrap();
        assert_eq!((icon.width(), icon.height()), (48, 48));
        assert_eq!(icon.pixels().len(), 48 * 48 * 4);
        assert_eq!(icon.pixels()[0], 0);
        assert_eq!(icon.pixels()[24 * 48 * 4], 255);
    }
}
