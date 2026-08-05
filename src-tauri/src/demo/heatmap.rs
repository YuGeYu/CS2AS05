use std::{fs, io::Cursor, path::Path};

use sha2::{Digest, Sha256};

use crate::{demo::map_metadata::MapMetadata, errors::AppError, models::demo::HeatmapPoint};

fn err(code: &str, detail: impl std::fmt::Display) -> AppError {
    AppError::runtime(format!("[{code}] {detail}"))
}
pub fn sha256(bytes: &[u8]) -> String {
    format!("{:X}", Sha256::digest(bytes))
}

pub fn decode_rgba(bytes: &[u8]) -> Result<(u32, u32, Vec<u8>), AppError> {
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder
        .read_info()
        .map_err(|e| err("DEMO_RADAR_DECODE", e))?;
    let mut raw = vec![0; reader.output_buffer_size()];
    let info = reader
        .next_frame(&mut raw)
        .map_err(|e| err("DEMO_RADAR_DECODE", e))?;
    let source = &raw[..info.buffer_size()];
    let rgba = match info.color_type {
        png::ColorType::Rgba => source.to_vec(),
        png::ColorType::Rgb => source
            .chunks_exact(3)
            .flat_map(|p| [p[0], p[1], p[2], 255])
            .collect(),
        _ => return Err(err("DEMO_RADAR_FORMAT", "radar must decode as RGB or RGBA")),
    };
    Ok((info.width, info.height, rgba))
}

pub fn render(
    metadata: &MapMetadata,
    radar: &[u8],
    points: &[HeatmapPoint],
    radius: u32,
    opacity: f64,
) -> Result<(Vec<u8>, usize, usize), AppError> {
    let (width, height, mut pixels) = decode_rgba(radar)?;
    if width != 1024 || height != 1024 || metadata.radar_size != 1024 {
        return Err(err(
            "DEMO_RADAR_SIZE",
            format!("expected 1024x1024 radar, got {width}x{height}"),
        ));
    }
    let mut density = vec![0_f32; (width * height) as usize];
    let mut rendered = 0;
    for point in points {
        if !point.x.is_finite() || !point.y.is_finite() {
            continue;
        }
        let (x, y) = metadata.scale_coordinate(f64::from(width), point.x, point.y);
        if !x.is_finite()
            || !y.is_finite()
            || x < 0.0
            || y < 0.0
            || x >= f64::from(width)
            || y >= f64::from(height)
        {
            continue;
        }
        rendered += 1;
        let cx = x.round() as i32;
        let cy = y.round() as i32;
        let radius = radius as i32;
        for oy in -radius..=radius {
            for ox in -radius..=radius {
                let distance = ((ox * ox + oy * oy) as f64).sqrt();
                if distance > f64::from(radius) {
                    continue;
                }
                let px = cx + ox;
                let py = cy + oy;
                if px < 0 || py < 0 || px >= width as i32 || py >= height as i32 {
                    continue;
                }
                let kernel = (1.0 - distance / f64::from(radius.max(1))).powi(2) as f32;
                density[(py as u32 * width + px as u32) as usize] += kernel * point.weight as f32;
            }
        }
    }
    let maximum = density.iter().copied().fold(0_f32, f32::max);
    if maximum > 0.0 {
        for (index, value) in density
            .into_iter()
            .enumerate()
            .filter(|(_, value)| *value > 0.0)
        {
            let amount = (value / maximum).clamp(0.0, 1.0);
            let (r, g, b) = if amount < 0.5 {
                let t = amount * 2.0;
                (
                    (59.0 + 196.0 * t) as u8,
                    (130.0 + 65.0 * t) as u8,
                    (246.0 - 198.0 * t) as u8,
                )
            } else {
                let t = (amount - 0.5) * 2.0;
                (255, (195.0 - 157.0 * t) as u8, (48.0 - 10.0 * t) as u8)
            };
            let alpha = (f64::from(amount) * opacity).clamp(0.0, 1.0) as f32;
            let pixel = &mut pixels[index * 4..index * 4 + 4];
            pixel[0] = (f32::from(pixel[0]) * (1.0 - alpha) + f32::from(r) * alpha).round() as u8;
            pixel[1] = (f32::from(pixel[1]) * (1.0 - alpha) + f32::from(g) * alpha).round() as u8;
            pixel[2] = (f32::from(pixel[2]) * (1.0 - alpha) + f32::from(b) * alpha).round() as u8;
            pixel[3] = 255;
        }
    }
    Ok((pixels, rendered, points.len() - rendered))
}

pub fn encode_rgba(path: &Path, pixels: &[u8]) -> Result<(), AppError> {
    let file = fs::File::create(path).map_err(|e| err("DEMO_EXPORT_WRITE", e))?;
    let mut encoder = png::Encoder::new(file, 1024, 1024);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()
        .map_err(|e| err("DEMO_EXPORT_WRITE", e))?
        .write_image_data(pixels)
        .map_err(|e| err("DEMO_EXPORT_WRITE", e))
}

#[cfg(windows)]
pub fn replace_file(source: &Path, destination: &Path) -> Result<(), AppError> {
    use std::os::windows::ffi::OsStrExt;
    use windows_sys::Win32::Storage::FileSystem::{
        MoveFileExW, MOVEFILE_REPLACE_EXISTING, MOVEFILE_WRITE_THROUGH,
    };
    let source: Vec<u16> = source.as_os_str().encode_wide().chain(Some(0)).collect();
    let destination: Vec<u16> = destination
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    if unsafe {
        MoveFileExW(
            source.as_ptr(),
            destination.as_ptr(),
            MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH,
        )
    } == 0
    {
        return Err(err("DEMO_EXPORT_REPLACE", std::io::Error::last_os_error()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::demo::map_metadata;

    fn point(x: f64, y: f64) -> HeatmapPoint {
        HeatmapPoint {
            tick: 100,
            x,
            y,
            z: Some(0.0),
            weight: 1.0,
            kind: "player_death".into(),
            round_number: Some(1),
            player_key: Some("bot:1".into()),
            team_number: Some(2),
        }
    }

    #[test]
    fn real_dust2_radar_renders_stable_overlay_and_discards_bad_points() {
        let maps = map_metadata::embedded().unwrap();
        let dust2 = maps.iter().find(|map| map.name == "de_dust2").unwrap();
        let radar = include_bytes!("../../resources/demo-maps/radars/de_dust2.png");
        assert_eq!(sha256(radar), dust2.radar_sha256);
        let center_x = dust2.position_x + dust2.scale * 512.0;
        let center_y = dust2.position_y - dust2.scale * 512.0;
        let points = vec![
            point(center_x, center_y),
            point(f64::NAN, 0.0),
            point(1_000_000.0, 1_000_000.0),
        ];
        let (_, _, original) = decode_rgba(radar).unwrap();
        let (rendered, accepted, discarded) = render(dust2, radar, &points, 18, 0.72).unwrap();
        assert_eq!((accepted, discarded), (1, 2));
        let changed = original
            .chunks_exact(4)
            .zip(rendered.chunks_exact(4))
            .filter(|(a, b)| a != b)
            .count();
        assert!(changed > 100 && changed < original.len() / 16);
        let mut reversed = points.clone();
        reversed.reverse();
        assert_eq!(
            render(dust2, radar, &reversed, 18, 0.72).unwrap().0,
            rendered
        );
    }

    #[test]
    fn double_layer_assets_match_embedded_hashes() {
        let maps = map_metadata::embedded().unwrap();
        let nuke = maps.iter().find(|map| map.name == "de_nuke").unwrap();
        let vertigo = maps.iter().find(|map| map.name == "de_vertigo").unwrap();
        assert_eq!(
            sha256(include_bytes!(
                "../../resources/demo-maps/radars/de_nuke_lower.png"
            )),
            nuke.lower_radar_sha256.as_deref().unwrap()
        );
        assert_eq!(
            sha256(include_bytes!(
                "../../resources/demo-maps/radars/de_vertigo_lower.png"
            )),
            vertigo.lower_radar_sha256.as_deref().unwrap()
        );
    }
}

#[cfg(not(windows))]
pub fn replace_file(source: &Path, destination: &Path) -> Result<(), AppError> {
    fs::rename(source, destination).map_err(|e| err("DEMO_EXPORT_REPLACE", e))
}
