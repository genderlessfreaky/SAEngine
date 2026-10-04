//! Leitor de modelos .dff (RenderWare "Clump") do San Andreas.
//!
//! Um modelo tem:
//!  - frames: as "pecas" (chassi, portas, rodas...), com nome, posicao e pai
//!  - geometrias: pontos, triangulos, UVs e materiais
//!  - atomics: ligam uma geometria a um frame (e isso que aparece na tela)
//!
//! As coordenadas ficam no padrao do SA (Z para cima). A conversao para o
//! Bevy (Y para cima) e feita na hora de montar a cena.

use super::stream::*;
use std::io::{self, ErrorKind};

#[derive(Debug, Clone)]
pub struct Frame {
    pub name: String,
    /// vetores direita, frente, cima (colunas da matriz de rotacao)
    pub right: [f32; 3],
    pub up: [f32; 3],
    pub at: [f32; 3],
    pub pos: [f32; 3],
    pub parent: i32,
}

#[derive(Debug, Clone)]
pub struct Material {
    pub color: [u8; 4],
    pub texture: Option<String>,
}

#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub a: u16,
    pub b: u16,
    pub c: u16,
    pub material: u16,
}

#[derive(Debug, Clone, Default)]
pub struct Geometry {
    pub positions: Vec<[f32; 3]>,
    pub normals: Vec<[f32; 3]>,
    pub uvs: Vec<[f32; 2]>,
    pub colors: Vec<[u8; 4]>,
    pub triangles: Vec<Triangle>,
    pub materials: Vec<Material>,
}

#[derive(Debug, Clone, Copy)]
pub struct Atomic {
    pub frame: usize,
    pub geometry: usize,
    pub flags: u32,
}

#[derive(Debug, Clone, Default)]
pub struct Clump {
    pub frames: Vec<Frame>,
    pub geometries: Vec<Geometry>,
    pub atomics: Vec<Atomic>,
}

impl Clump {
    pub fn frame_by_name(&self, name: &str) -> Option<usize> {
        self.frames.iter().position(|f| f.name.eq_ignore_ascii_case(name))
    }
}

fn bad(msg: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, msg.to_string())
}

pub fn parse_dff(data: &[u8]) -> io::Result<Clump> {
    let mut r = Reader::new(data);
    // procura o primeiro Clump (alguns arquivos tem outros blocos antes)
    while r.remaining() >= 12 {
        let c = r.chunk()?;
        if c.kind == CH_CLUMP {
            return parse_clump(&mut r, c);
        }
        r.goto(c.end());
    }
    Err(bad("nenhum Clump encontrado no .dff"))
}

fn parse_clump(r: &mut Reader, clump: Chunk) -> io::Result<Clump> {
    let mut out = Clump::default();
    while r.pos + 12 <= clump.end() {
        let c = r.chunk()?;
        match c.kind {
            CH_FRAME_LIST => out.frames = parse_frame_list(r, c)?,
            CH_GEOMETRY_LIST => out.geometries = parse_geometry_list(r, c)?,
            CH_ATOMIC => {
                let s = r.expect(CH_STRUCT)?;
                let frame = r.u32()? as usize;
                let geometry = r.u32()? as usize;
                let flags = r.u32()?;
                r.goto(s.end());
                out.atomics.push(Atomic { frame, geometry, flags });
            }
            _ => {}
        }
        r.goto(c.end());
    }
    Ok(out)
}

fn parse_frame_list(r: &mut Reader, list: Chunk) -> io::Result<Vec<Frame>> {
    let s = r.expect(CH_STRUCT)?;
    let count = r.u32()? as usize;
    let mut frames = Vec::with_capacity(count);
    for _ in 0..count {
        let right = r.vec3()?;
        let up = r.vec3()?;
        let at = r.vec3()?;
        let pos = r.vec3()?;
        let parent = r.i32()?;
        r.skip(4)?;
        frames.push(Frame { name: String::new(), right, up, at, pos, parent });
    }
    r.goto(s.end());

    // depois vem um bloco Extension por frame, que pode ter o nome
    let mut i = 0;
    while r.pos + 12 <= list.end() && i < count {
        let ext = r.chunk()?;
        if ext.kind == CH_EXTENSION {
            while r.pos + 12 <= ext.end() {
                let e = r.chunk()?;
                if e.kind == CH_FRAME_NAME {
                    frames[i].name = c_str(&r.data[e.start..e.end()]);
                }
                r.goto(e.end());
            }
            i += 1;
        }
        r.goto(ext.end());
    }
    Ok(frames)
}

fn parse_geometry_list(r: &mut Reader, list: Chunk) -> io::Result<Vec<Geometry>> {
    let s = r.expect(CH_STRUCT)?;
    let count = r.u32()? as usize;
    r.goto(s.end());
    let mut out = Vec::with_capacity(count);
    while r.pos + 12 <= list.end() && out.len() < count {
        let g = r.chunk()?;
        if g.kind == CH_GEOMETRY {
            out.push(parse_geometry(r, g)?);
        }
        r.goto(g.end());
    }
    Ok(out)
}

fn parse_geometry(r: &mut Reader, geo: Chunk) -> io::Result<Geometry> {
    let version = unpack_version(geo.version);
    let s = r.expect(CH_STRUCT)?;
    let format = r.u32()?;
    let flags = format & 0xFFFF;
    let mut num_uv = ((format >> 16) & 0xFF) as usize;
    let native = (format >> 24) & 0xFF != 0;
    let tri_count = r.u32()? as usize;
    let vert_count = r.u32()? as usize;
    let _morph_count = r.u32()?;
    if version < 0x34000 {
        r.skip(12)?;
    }

    let mut g = Geometry::default();
    if !native {
        if num_uv == 0 {
            if flags & 0x80 != 0 {
                num_uv = 2;
            } else if flags & 0x04 != 0 {
                num_uv = 1;
            }
        }
        if flags & 0x08 != 0 {
            for _ in 0..vert_count {
                let b = r.bytes(4)?;
                g.colors.push([b[0], b[1], b[2], b[3]]);
            }
        }
        for set in 0..num_uv {
            for _ in 0..vert_count {
                let u = r.f32()?;
                let v = r.f32()?;
                if set == 0 {
                    g.uvs.push([u, v]);
                }
            }
        }
        for _ in 0..tri_count {
            let b = r.u16()?;
            let a = r.u16()?;
            let material = r.u16()?;
            let c = r.u16()?;
            g.triangles.push(Triangle { a, b, c, material });
        }
        // primeiro "morph target": esfera, tem vertices?, tem normais?
        r.skip(16)?;
        let has_verts = r.u32()? != 0;
        let has_normals = r.u32()? != 0;
        if has_verts {
            for _ in 0..vert_count {
                g.positions.push(r.vec3()?);
            }
        }
        if has_normals {
            for _ in 0..vert_count {
                g.normals.push(r.vec3()?);
            }
        }
    }
    r.goto(s.end());

    while r.pos + 12 <= geo.end() {
        let c = r.chunk()?;
        if c.kind == CH_MATERIAL_LIST {
            g.materials = parse_material_list(r, c)?;
        }
        r.goto(c.end());
    }
    Ok(g)
}

fn parse_material_list(r: &mut Reader, list: Chunk) -> io::Result<Vec<Material>> {
    let s = r.expect(CH_STRUCT)?;
    let count = r.u32()? as usize;
    let mut refs = Vec::with_capacity(count);
    for _ in 0..count {
        refs.push(r.i32()?);
    }
    r.goto(s.end());

    let mut unique = Vec::new();
    while r.pos + 12 <= list.end() {
        let c = r.chunk()?;
        if c.kind == CH_MATERIAL {
            unique.push(parse_material(r, c)?);
        }
        r.goto(c.end());
    }

    // -1 = material novo (na ordem), outro numero = repete um anterior
    let mut out = Vec::with_capacity(count);
    let mut next = 0;
    for &rf in &refs {
        if rf < 0 {
            let m = unique.get(next).cloned().unwrap_or(Material { color: [255; 4], texture: None });
            next += 1;
            out.push(m);
        } else {
            let m = out.get(rf as usize).cloned().unwrap_or(Material { color: [255; 4], texture: None });
            out.push(m);
        }
    }
    Ok(out)
}

fn parse_material(r: &mut Reader, mat: Chunk) -> io::Result<Material> {
    let s = r.expect(CH_STRUCT)?;
    let _flags = r.u32()?;
    let b = r.bytes(4)?;
    let color = [b[0], b[1], b[2], b[3]];
    let _unused = r.u32()?;
    let textured = r.u32()? != 0;
    r.goto(s.end());

    let mut texture = None;
    if textured {
        while r.pos + 12 <= mat.end() {
            let c = r.chunk()?;
            if c.kind == CH_TEXTURE {
                let ts = r.expect(CH_STRUCT)?;
                r.goto(ts.end());
                let name = r.expect(CH_STRING)?;
                texture = Some(c_str(&r.data[name.start..name.end()]));
                r.goto(c.end());
                break;
            }
            r.goto(c.end());
        }
    }
    Ok(Material { color, texture })
}
