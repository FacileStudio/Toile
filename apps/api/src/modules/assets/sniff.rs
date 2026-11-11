use imagesize::ImageType;
use tronc::{Error, Result};

/// What the bytes of an upload turned out to be.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Image {
    /// The media type served back.
    pub mime: &'static str,
    /// The file extension in Nuage.
    pub ext: &'static str,
    /// Width in pixels.
    pub width: u32,
    /// Height in pixels.
    pub height: u32,
}

fn kind(bytes: &[u8]) -> Option<(&'static str, &'static str)> {
    match imagesize::image_type(bytes).ok()? {
        ImageType::Png => Some(("image/png", "png")),
        ImageType::Jpeg => Some(("image/jpeg", "jpg")),
        ImageType::Gif => Some(("image/gif", "gif")),
        ImageType::Webp => Some(("image/webp", "webp")),
        _ => None,
    }
}

fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = String::from_utf8_lossy(&bytes[..bytes.len().min(1024)]).to_ascii_lowercase();
    head.contains("<svg")
}

/// Reads the type from the bytes themselves, never from the name or the declared type: PNG,
/// JPEG, GIF and WebP pass with their size; SVG is refused (it can carry script), as is
/// anything else.
pub fn sniff(bytes: &[u8]) -> Result<Image> {
    let Some((mime, ext)) = kind(bytes) else {
        return Err(Error::invalid(match looks_like_svg(bytes) {
            true => "SVG images are not accepted: export the image as PNG",
            false => "only PNG, JPEG, GIF and WebP images are accepted",
        }));
    };
    let size = imagesize::blob_size(bytes).map_err(|_| Error::invalid("the image is damaged"))?;
    let (width, height) = (u32::try_from(size.width).unwrap_or(0), u32::try_from(size.height).unwrap_or(0));
    if width == 0 || height == 0 {
        return Err(Error::invalid("the image is damaged"));
    }
    Ok(Image { mime, ext, width, height })
}

#[cfg(test)]
mod tests {
    use super::*;

    const PNG: &[u8] = &[
        0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a, 0, 0, 0, 13, b'I', b'H', b'D', b'R', 0, 0, 0, 3, 0, 0, 0, 2, 8,
        6, 0, 0, 0,
    ];

    #[test]
    fn the_bytes_decide_the_type_and_svg_is_refused() {
        assert_eq!(sniff(PNG).unwrap(), Image { mime: "image/png", ext: "png", width: 3, height: 2 });
        let svg = sniff(br#"<?xml version="1.0"?><svg xmlns="http://www.w3.org/2000/svg"><script/></svg>"#);
        assert!(svg.unwrap_err().message.contains("SVG"));
        assert!(sniff(b"%PDF-1.7").unwrap_err().message.contains("PNG, JPEG"));
        assert!(sniff(&PNG[..12]).is_err());
    }
}
