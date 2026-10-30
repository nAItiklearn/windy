//! Per-OS integration for Windy. Windows is implemented natively; macOS and Linux use the
//! portable paths (arboard, enigo) and are compile-checked by CI.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use windy_core::clipboard::CopyEvent;
use windy_core::types::kind;
use windy_core::Content;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("clipboard: {0}")]
    Clipboard(String),
    #[error("paste injection: {0}")]
    Input(String),
}

pub type Result<T> = std::result::Result<T, Error>;

fn cb_err(e: arboard::Error) -> Error {
    Error::Clipboard(e.to_string())
}

#[cfg(not(windows))]
fn hash_bytes(parts: &[&[u8]]) -> u64 {
    let mut h = DefaultHasher::new();
    for p in parts {
        p.hash(&mut h);
    }
    h.finish()
}

/// A value that changes whenever the clipboard content changes.
/// Windows has a real sequence number; elsewhere this hashes the visible content.
pub fn change_token() -> u64 {
    #[cfg(windows)]
    {
        windows_impl::sequence_number()
    }
    #[cfg(not(windows))]
    {
        let Ok(mut cb) = arboard::Clipboard::new() else {
            return 0;
        };
        if let Ok(t) = cb.get_text() {
            return hash_bytes(&[b"t", t.as_bytes()]);
        }
        if let Ok(img) = cb.get_image() {
            return hash_bytes(&[b"i", &img.bytes]);
        }
        0
    }
}

/// Stable hash of the contents Windy writes, used to recognise its own copies.
pub fn contents_fingerprint(contents: &[Content]) -> u64 {
    let mut h = DefaultHasher::new();
    for c in contents {
        c.kind.hash(&mut h);
        c.data.hash(&mut h);
    }
    h.finish()
}

fn encode_png(width: usize, height: usize, rgba: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut enc = png::Encoder::new(
        &mut out,
        u32::try_from(width).ok()?,
        u32::try_from(height).ok()?,
    );
    enc.set_color(png::ColorType::Rgba);
    enc.set_depth(png::BitDepth::Eight);
    enc.write_header().ok()?.write_image_data(rgba).ok()?;
    Some(out)
}

fn decode_png(data: &[u8]) -> Option<(usize, usize, Vec<u8>)> {
    let mut dec = png::Decoder::new(data);
    dec.set_transformations(png::Transformations::EXPAND);
    let mut reader = dec.read_info().ok()?;
    let mut buf = vec![0; reader.output_buffer_size()];
    let info = reader.next_frame(&mut buf).ok()?;
    buf.truncate(info.buffer_size());
    let rgba = match info.color_type {
        png::ColorType::Rgba => buf,
        png::ColorType::Rgb => buf
            .chunks(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        png::ColorType::GrayscaleAlpha => buf
            .chunks(2)
            .flat_map(|p| [p[0], p[0], p[0], p[1]])
            .collect(),
        png::ColorType::Grayscale => buf.iter().flat_map(|g| [*g, *g, *g, 255]).collect(),
        png::ColorType::Indexed => return None,
    };
    Some((info.width as usize, info.height as usize, rgba))
}

/// Reads the current clipboard into a `CopyEvent`.
pub fn read_event() -> CopyEvent {
    let mut items: Vec<Vec<Content>> = Vec::new();
    if let Ok(mut cb) = arboard::Clipboard::new() {
        let mut item = Vec::new();
        if let Ok(t) = cb.get_text() {
            item.push(Content::text(&t));
        }
        if let Ok(h) = cb.get().html() {
            item.push(Content::new(kind::HTML, h));
        }
        if let Ok(files) = cb.get().file_list() {
            let uris: Vec<String> = files
                .iter()
                .map(|p| path_to_uri(&p.to_string_lossy()))
                .collect();
            if !uris.is_empty() {
                item.push(Content::new(kind::FILES, uris.join("\n")));
            }
        }
        if item.iter().all(|c| c.kind != kind::TEXT) {
            if let Ok(img) = cb.get_image() {
                if let Some(png) = encode_png(img.width, img.height, &img.bytes) {
                    item.push(Content::new(kind::PNG, png));
                }
            }
        }
        if !item.is_empty() {
            items.push(item);
        }
    }
    #[cfg(windows)]
    if windows_impl::is_sensitive() {
        items.push(vec![Content::new(kind::CONCEALED, "1")]);
    }
    CopyEvent {
        items,
        frontmost_app: frontmost_app(),
        from_windy: false,
    }
}

fn path_to_uri(path: &str) -> String {
    let mut out = String::from("file:///");
    for b in path.replace('\\', "/").trim_start_matches('/').bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b':' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

/// Replaces the clipboard with the given contents.
pub fn write_contents(contents: &[Content]) -> Result<()> {
    let mut cb = arboard::Clipboard::new().map_err(cb_err)?;
    let text = contents
        .iter()
        .find(|c| c.kind == kind::TEXT)
        .and_then(Content::as_str);
    let html = contents
        .iter()
        .find(|c| c.kind == kind::HTML)
        .and_then(Content::as_str);
    if let Some(h) = html {
        return cb.set_html(h, text).map_err(cb_err);
    }
    if let Some(t) = text {
        return cb.set_text(t).map_err(cb_err);
    }
    if let Some(png) = contents.iter().find(|c| c.kind == kind::PNG) {
        let (w, h, rgba) =
            decode_png(&png.data).ok_or_else(|| Error::Clipboard("bad image".into()))?;
        return cb
            .set_image(arboard::ImageData {
                width: w,
                height: h,
                bytes: rgba.into(),
            })
            .map_err(cb_err);
    }
    if let Some(files) = contents
        .iter()
        .find(|c| c.kind == kind::FILES)
        .and_then(Content::as_str)
    {
        return cb.set_text(files).map_err(cb_err);
    }
    Err(Error::Clipboard("nothing to write".into()))
}

pub fn clear_clipboard() -> Result<()> {
    arboard::Clipboard::new()
        .map_err(cb_err)?
        .clear()
        .map_err(cb_err)
}

/// Sends the system Paste shortcut to whichever window has focus.
pub fn send_paste() -> Result<()> {
    use enigo::{Direction, Enigo, Key, Keyboard, Settings};
    let mut e = Enigo::new(&Settings::default()).map_err(|e| Error::Input(e.to_string()))?;
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;
    let err = |e: enigo::InputError| Error::Input(e.to_string());
    e.key(modifier, Direction::Press).map_err(err)?;
    let r = e.key(Key::Unicode('v'), Direction::Click).map_err(err);
    e.key(modifier, Direction::Release).map_err(err)?;
    r
}

/// Identifier of the application that currently has focus (executable name on Windows).
pub fn frontmost_app() -> Option<String> {
    #[cfg(windows)]
    {
        windows_impl::foreground_exe()
    }
    #[cfg(not(windows))]
    {
        None
    }
}

#[cfg(windows)]
#[allow(unsafe_code)]
mod windows_impl {
    use windows_sys::Win32::Foundation::CloseHandle;
    use windows_sys::Win32::System::DataExchange::{
        GetClipboardSequenceNumber, IsClipboardFormatAvailable, RegisterClipboardFormatW,
    };
    use windows_sys::Win32::System::Threading::{
        OpenProcess, QueryFullProcessImageNameW, PROCESS_QUERY_LIMITED_INFORMATION,
    };
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        GetForegroundWindow, GetWindowThreadProcessId,
    };

    pub fn sequence_number() -> u64 {
        u64::from(unsafe { GetClipboardSequenceNumber() })
    }

    fn wide(s: &str) -> Vec<u16> {
        s.encode_utf16().chain(std::iter::once(0)).collect()
    }

    /// Password managers mark secrets with this registered clipboard format.
    pub fn is_sensitive() -> bool {
        let id = unsafe {
            RegisterClipboardFormatW(wide("ExcludeClipboardContentFromMonitorProcessing").as_ptr())
        };
        id != 0 && unsafe { IsClipboardFormatAvailable(id) } != 0
    }

    pub fn foreground_exe() -> Option<String> {
        unsafe {
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() {
                return None;
            }
            let mut pid = 0u32;
            GetWindowThreadProcessId(hwnd, &mut pid);
            let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if h.is_null() {
                return None;
            }
            let mut buf = [0u16; 520];
            let mut len = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(h, 0, buf.as_mut_ptr(), &mut len);
            CloseHandle(h);
            if ok == 0 {
                return None;
            }
            let path = String::from_utf16_lossy(&buf[..len as usize]);
            path.rsplit(['\\', '/']).next().map(str::to_lowercase)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn png_round_trip() {
        let rgba: Vec<u8> = (0..4 * 4 * 4).map(|i| i as u8).collect();
        let png = encode_png(4, 4, &rgba).unwrap();
        let (w, h, back) = decode_png(&png).unwrap();
        assert_eq!((w, h), (4, 4));
        assert_eq!(back, rgba);
    }

    #[test]
    fn file_uris_are_percent_encoded() {
        assert_eq!(
            path_to_uri(r"C:\My Docs\é.txt"),
            "file:///C:/My%20Docs/%C3%A9.txt"
        );
    }

    #[test]
    fn fingerprint_depends_on_content() {
        let a = [Content::text("a")];
        let b = [Content::text("b")];
        assert_eq!(contents_fingerprint(&a), contents_fingerprint(&a));
        assert_ne!(contents_fingerprint(&a), contents_fingerprint(&b));
    }
}
