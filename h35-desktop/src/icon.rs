use std::io::Cursor;
use std::sync::OnceLock;

static ICON_PNG: OnceLock<Option<&'static [u8]>> = OnceLock::new();

#[derive(Debug)]
pub(crate) struct RgbaIcon {
    pub rgba: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub(crate) fn set_icon_png(bytes: Option<&'static [u8]>) {
    let _ = ICON_PNG.set(bytes);
}

fn icon_png() -> Option<&'static [u8]> {
    ICON_PNG.get().copied().flatten()
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
            .as_chunks::<3>()
            .0
            .iter()
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
    let bytes = icon_png()?;
    match decode_rgba(bytes) {
        Ok(icon) => match tao::window::Icon::from_rgba(icon.rgba, icon.width, icon.height) {
            Ok(icon) => Some(icon),
            Err(error) => {
                tracing::error!(%error, "failed to build window icon");
                None
            }
        },
        Err(error) => {
            tracing::error!(error, "failed to decode host icon");
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

    let Some(bytes) = icon_png() else {
        return;
    };
    let Some(mtm) = MainThreadMarker::new() else {
        tracing::warn!("skipped dock icon: not on the main thread");
        return;
    };
    let data = NSData::with_bytes(bytes);
    let Some(image) = NSImage::initWithData(NSImage::alloc(), &data) else {
        tracing::error!("failed to load dock icon");
        return;
    };
    let app = NSApplication::sharedApplication(mtm);
    unsafe {
        app.setApplicationIconImage(Some(&image));
    }
    app.dockTile().display();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decodes_minimal_rgba_png() {
        let png = minimal_rgba_png();
        let icon = decode_rgba(&png).expect("decode png");
        assert_eq!(icon.width, 1);
        assert_eq!(icon.height, 1);
        assert_eq!(icon.rgba.len(), 4);
        tao::window::Icon::from_rgba(icon.rgba, icon.width, icon.height).expect("window icon");
    }

    fn minimal_rgba_png() -> Vec<u8> {
        let mut out = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut out, 1, 1);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(&[255, 0, 0, 255]).unwrap();
        }
        out
    }
}
