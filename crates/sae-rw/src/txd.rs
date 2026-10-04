//! Leitor de texturas .txd (versao PC do San Andreas / D3D8 e D3D9).
//!
//! Cada textura e convertida para RGBA comum (4 bytes por pixel), pronta
//! para virar imagem no Bevy. Formatos suportados: DXT1/3/5 (comprimidos),
//! 8888, 888, 565, 1555, 555, 4444, LUM8 e paleta de 8 ou 4 bits.

use super::stream::*;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone)]
pub struct Texture {
    pub name: String,
    pub width: u32,
    pub height: u32,
    /// pixels RGBA, linha por linha
    pub rgba: Vec<u8>,
    /// true se algum pixel tem transparencia
    pub has_alpha: bool,
}

const FMT_1555: u32 = 0x100;
const FMT_565: u32 = 0x200;
const FMT_4444: u32 = 0x300;
const FMT_LUM8: u32 = 0x400;
const FMT_8888: u32 = 0x500;
const FMT_888: u32 = 0x600;
const FMT_555: u32 = 0xA00;
const FMT_PAL8: u32 = 0x2000;
const FMT_PAL4: u32 = 0x4000;

pub fn parse_txd(data: &[u8]) -> io::Result<Vec<Texture>> {
    let mut r = Reader::new(data);
    let dict = r.expect(CH_TEX_DICTIONARY)?;
    let s = r.expect(CH_STRUCT)?;
    r.goto(s.end());
    let mut out = Vec::new();
    while r.pos + 12 <= dict.end() {
        let c = r.chunk()?;
        if c.kind == CH_TEXTURE_NATIVE {
            match parse_native(&mut r) {
                Ok(Some(t)) => out.push(t),
                Ok(None) => {}
                Err(e) => log::warn!("textura ignorada: {e}"),
            }
        }
        r.goto(c.end());
    }
    Ok(out)
}

fn parse_native(r: &mut Reader) -> io::Result<Option<Texture>> {
    let s = r.expect(CH_STRUCT)?;
    let platform = r.u32()?;
    if platform != 8 && platform != 9 {
        r.goto(s.end());
        return Ok(None); // PS2/Xbox: nao e da versao PC
    }
    let _filter = r.u32()?;
    let name = r.fixed_str(32)?;
    let _mask = r.fixed_str(32)?;
    let raster = r.u32()?;
    let d3d = r.bytes(4)?;
    let fourcc = [d3d[0], d3d[1], d3d[2], d3d[3]];
    let width = r.u16()? as u32;
    let height = r.u16()? as u32;
    let _depth = r.u8()?;
    let _levels = r.u8()?;
    let _raster_type = r.u8()?;
    let comp = r.u8()?;

    let palette: Vec<u8> = if raster & FMT_PAL8 != 0 {
        r.bytes(256 * 4)?.to_vec()
    } else if raster & FMT_PAL4 != 0 {
        r.bytes(16 * 4)?.to_vec()
    } else {
        Vec::new()
    };

    let size = r.u32()? as usize;
    let px = r.bytes(size)?;

    // Descobre se e comprimida (DXT) e qual tipo
    let dxt: Option<u8> = if platform == 9 {
        if comp & 0x08 != 0 {
            match &fourcc {
                b"DXT1" => Some(1),
                b"DXT2" | b"DXT3" => Some(3),
                b"DXT4" | b"DXT5" => Some(5),
                _ => None,
            }
        } else {
            None
        }
    } else if comp != 0 {
        Some(comp) // D3D8: 1 = DXT1, 3 = DXT3
    } else {
        None
    };

    let n = (width * height) as usize;
    let rgba = if let Some(kind) = dxt {
        decode_dxt(kind, px, width, height)
    } else if !palette.is_empty() {
        let mut v = Vec::with_capacity(n * 4);
        for i in 0..n {
            let idx = *px.get(i).unwrap_or(&0) as usize;
            let p = palette.get(idx * 4..idx * 4 + 4).unwrap_or(&[255, 0, 255, 255]);
            v.extend_from_slice(p);
        }
        v
    } else {
        decode_raw(raster & 0xF00, px, n)
    };

    if rgba.len() != n * 4 {
        return Err(io::Error::new(
            ErrorKind::InvalidData,
            format!("textura '{name}' com formato nao suportado (0x{raster:X})"),
        ));
    }
    let has_alpha = rgba.chunks_exact(4).any(|p| p[3] < 250);
    r.goto(s.end());
    Ok(Some(Texture { name, width, height, rgba, has_alpha }))
}

fn decode_raw(fmt: u32, px: &[u8], n: usize) -> Vec<u8> {
    let mut v = Vec::with_capacity(n * 4);
    let w16 = |i: usize| -> u32 { u16::from_le_bytes([px[i * 2], px[i * 2 + 1]]) as u32 };
    let x5 = |c: u32| ((c << 3) | (c >> 2)) as u8;
    let x6 = |c: u32| ((c << 2) | (c >> 4)) as u8;
    let x4 = |c: u32| ((c << 4) | c) as u8;
    match fmt {
        FMT_8888 | FMT_888 if px.len() >= n * 4 => {
            for i in 0..n {
                let b = &px[i * 4..i * 4 + 4];
                let a = if fmt == FMT_888 { 255 } else { b[3] };
                v.extend_from_slice(&[b[2], b[1], b[0], a]);
            }
        }
        FMT_565 if px.len() >= n * 2 => {
            for i in 0..n {
                let c = w16(i);
                v.extend_from_slice(&[x5(c >> 11), x6((c >> 5) & 63), x5(c & 31), 255]);
            }
        }
        FMT_1555 | FMT_555 if px.len() >= n * 2 => {
            for i in 0..n {
                let c = w16(i);
                let a = if fmt == FMT_555 || c & 0x8000 != 0 { 255 } else { 0 };
                v.extend_from_slice(&[x5((c >> 10) & 31), x5((c >> 5) & 31), x5(c & 31), a]);
            }
        }
        FMT_4444 if px.len() >= n * 2 => {
            for i in 0..n {
                let c = w16(i);
                v.extend_from_slice(&[x4((c >> 8) & 15), x4((c >> 4) & 15), x4(c & 15), x4(c >> 12)]);
            }
        }
        FMT_LUM8 if px.len() >= n => {
            for &l in &px[..n] {
                v.extend_from_slice(&[l, l, l, 255]);
            }
        }
        _ => {}
    }
    v
}

fn rgb565(c: u16) -> [u8; 3] {
    let r = ((c >> 11) & 31) as u32;
    let g = ((c >> 5) & 63) as u32;
    let b = (c & 31) as u32;
    [((r << 3) | (r >> 2)) as u8, ((g << 2) | (g >> 4)) as u8, ((b << 3) | (b >> 2)) as u8]
}

/// Descomprime DXT1, DXT3 ou DXT5 (blocos de 4x4 pixels).
fn decode_dxt(kind: u8, src: &[u8], w: u32, h: u32) -> Vec<u8> {
    let (w, h) = (w as usize, h as usize);
    let mut out = vec![0u8; w * h * 4];
    let block_bytes = if kind == 1 { 8 } else { 16 };
    let bw = w.div_ceil(4);
    let bh = h.div_ceil(4);
    if src.len() < bw * bh * block_bytes {
        return Vec::new();
    }
    for by in 0..bh {
        for bx in 0..bw {
            let b = &src[(by * bw + bx) * block_bytes..][..block_bytes];
            let (alpha_part, color_part) = if kind == 1 { (&b[0..0], &b[0..8]) } else { (&b[0..8], &b[8..16]) };

            let c0 = u16::from_le_bytes([color_part[0], color_part[1]]);
            let c1 = u16::from_le_bytes([color_part[2], color_part[3]]);
            let p0 = rgb565(c0);
            let p1 = rgb565(c1);
            let mut pal = [[0u8; 4]; 4];
            pal[0] = [p0[0], p0[1], p0[2], 255];
            pal[1] = [p1[0], p1[1], p1[2], 255];
            let mix = |a: u8, b: u8, wa: u32, wb: u32| ((a as u32 * wa + b as u32 * wb) / (wa + wb)) as u8;
            if c0 > c1 || kind != 1 {
                for k in 0..3 {
                    pal[2][k] = mix(p0[k], p1[k], 2, 1);
                    pal[3][k] = mix(p0[k], p1[k], 1, 2);
                }
                pal[2][3] = 255;
                pal[3][3] = 255;
            } else {
                for k in 0..3 {
                    pal[2][k] = mix(p0[k], p1[k], 1, 1);
                }
                pal[2][3] = 255;
                pal[3] = [0, 0, 0, 0];
            }
            let bits = u32::from_le_bytes([color_part[4], color_part[5], color_part[6], color_part[7]]);

            // alfa (DXT3: 4 bits por pixel; DXT5: interpolado)
            let mut alphas = [255u8; 16];
            if kind == 3 {
                for i in 0..16 {
                    let v = (alpha_part[i / 2] >> ((i % 2) * 4)) & 15;
                    alphas[i] = v * 17;
                }
            } else if kind == 5 {
                let a0 = alpha_part[0] as u32;
                let a1 = alpha_part[1] as u32;
                let mut ap = [0u32; 8];
                ap[0] = a0;
                ap[1] = a1;
                if a0 > a1 {
                    for i in 1..7 {
                        ap[i + 1] = ((7 - i as u32) * a0 + i as u32 * a1) / 7;
                    }
                } else {
                    for i in 1..5 {
                        ap[i + 1] = ((5 - i as u32) * a0 + i as u32 * a1) / 5;
                    }
                    ap[6] = 0;
                    ap[7] = 255;
                }
                let mut abits: u64 = 0;
                for i in 0..6 {
                    abits |= (alpha_part[2 + i] as u64) << (8 * i);
                }
                for i in 0..16 {
                    alphas[i] = ap[((abits >> (3 * i)) & 7) as usize] as u8;
                }
            }

            for py in 0..4 {
                for pxl in 0..4 {
                    let x = bx * 4 + pxl;
                    let y = by * 4 + py;
                    if x >= w || y >= h {
                        continue;
                    }
                    let i = py * 4 + pxl;
                    let idx = ((bits >> (2 * i)) & 3) as usize;
                    let mut c = pal[idx];
                    if kind != 1 {
                        c[3] = alphas[i];
                    }
                    out[(y * w + x) * 4..][..4].copy_from_slice(&c);
                }
            }
        }
    }
    out
}
