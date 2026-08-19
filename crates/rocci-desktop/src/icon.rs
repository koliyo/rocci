use std::io::Cursor;

const PNG: &[u8] = include_bytes!("../assets/rocci-icon.png");

#[derive(Debug)]
pub(crate) struct RgbaIcon {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub(crate) fn decode_rgba(bytes: &[u8]) -> Result<RgbaIcon, String> {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().map_err(|error| error.to_string())?;
    let mut buf = vec![
        0;
        reader.output_buffer_size().ok_or_else(|| {
            "png output is larger than addressable memory".to_string()
        })?
    ];
    let info = reader
        .next_frame(&mut buf)
        .map_err(|error| error.to_string())?;
    let pixels = &buf[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => pixels.to_vec(),
        png::ColorType::Rgb => pixels
            .chunks_exact(3)
            .flat_map(|pixel| [pixel[0], pixel[1], pixel[2], 255])
            .collect(),
        other => {
            return Err(format!(
                "unsupported png color type {other:?}; expected rgb or rgba"
            ));
        }
    };
    Ok(RgbaIcon {
        rgba,
        width: info.width,
        height: info.height,
    })
}

pub(crate) fn window_icon() -> Option<tao::window::Icon> {
    match decode_rgba(PNG) {
        Ok(icon) => match tao::window::Icon::from_rgba(icon.rgba, icon.width, icon.height) {
            Ok(icon) => Some(icon),
            Err(error) => {
                tracing::error!(%error, "failed to build window icon");
                None
            }
        },
        Err(error) => {
            tracing::error!(error, "failed to decode rocci icon");
            None
        }
    }
}

pub(crate) fn apply_host_icon() {
    #[cfg(target_os = "macos")]
    apply_macos_dock_icon();
}

#[cfg(target_os = "macos")]
fn apply_macos_dock_icon() {
    use objc2::{AllocAnyThread, MainThreadMarker};
    use objc2_app_kit::{NSApplication, NSImage};
    use objc2_foundation::NSData;

    let Some(mtm) = MainThreadMarker::new() else {
        tracing::warn!("skipped dock icon: not on the main thread");
        return;
    };
    let data = NSData::with_bytes(PNG);
    let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) else {
        tracing::error!("failed to load rocci dock icon");
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    unsafe {
        app.setApplicationIconImage(Some(&image));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_icon_is_1024_png() {
        assert!(PNG.starts_with(b"\x89PNG\r\n\x1a\n"));
        let icon = decode_rgba(PNG).expect("decode rocci icon png");
        assert_eq!(icon.width, 1024);
        assert_eq!(icon.height, 1024);
        assert_eq!(icon.rgba.len(), 1024 * 1024 * 4);
        tao::window::Icon::from_rgba(icon.rgba, icon.width, icon.height)
            .expect("window icon from rgba");
    }
}
