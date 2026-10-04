//! Leitura basica de "chunks" do RenderWare.
//!
//! Todo arquivo .dff e .txd e uma arvore de blocos. Cada bloco comeca com
//! 12 bytes: tipo (u32), tamanho (u32), versao (u32), seguido do conteudo.

use std::io::{self, ErrorKind};

pub const CH_STRUCT: u32 = 0x01;
pub const CH_STRING: u32 = 0x02;
pub const CH_EXTENSION: u32 = 0x03;
pub const CH_TEXTURE: u32 = 0x06;
pub const CH_MATERIAL: u32 = 0x07;
pub const CH_MATERIAL_LIST: u32 = 0x08;
pub const CH_FRAME_LIST: u32 = 0x0E;
pub const CH_GEOMETRY: u32 = 0x0F;
pub const CH_CLUMP: u32 = 0x10;
pub const CH_ATOMIC: u32 = 0x14;
pub const CH_TEXTURE_NATIVE: u32 = 0x15;
pub const CH_TEX_DICTIONARY: u32 = 0x16;
pub const CH_GEOMETRY_LIST: u32 = 0x1A;
pub const CH_FRAME_NAME: u32 = 0x0253_F2FE;

#[derive(Debug, Clone, Copy)]
pub struct Chunk {
    pub kind: u32,
    pub size: usize,
    pub version: u32,
    /// posicao do inicio do conteudo
    pub start: usize,
}

impl Chunk {
    pub fn end(&self) -> usize {
        self.start + self.size
    }
}

/// Converte o campo "library id" do RenderWare na versao numerica (ex: 0x36003).
pub fn unpack_version(lib_id: u32) -> u32 {
    if lib_id & 0xFFFF_0000 != 0 {
        ((lib_id >> 14) & 0x3_FF00) + 0x3_0000 | ((lib_id >> 16) & 0x3F)
    } else {
        lib_id << 8
    }
}

pub struct Reader<'a> {
    pub data: &'a [u8],
    pub pos: usize,
}

fn eof() -> io::Error {
    io::Error::new(ErrorKind::UnexpectedEof, "arquivo terminou antes do esperado")
}

impl<'a> Reader<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, pos: 0 }
    }

    pub fn remaining(&self) -> usize {
        self.data.len().saturating_sub(self.pos)
    }

    pub fn bytes(&mut self, n: usize) -> io::Result<&'a [u8]> {
        if self.pos + n > self.data.len() {
            return Err(eof());
        }
        let s = &self.data[self.pos..self.pos + n];
        self.pos += n;
        Ok(s)
    }

    pub fn skip(&mut self, n: usize) -> io::Result<()> {
        self.bytes(n).map(|_| ())
    }

    pub fn u8(&mut self) -> io::Result<u8> {
        Ok(self.bytes(1)?[0])
    }
    pub fn u16(&mut self) -> io::Result<u16> {
        let b = self.bytes(2)?;
        Ok(u16::from_le_bytes([b[0], b[1]]))
    }
    pub fn u32(&mut self) -> io::Result<u32> {
        let b = self.bytes(4)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }
    pub fn i32(&mut self) -> io::Result<i32> {
        Ok(self.u32()? as i32)
    }
    pub fn f32(&mut self) -> io::Result<f32> {
        Ok(f32::from_bits(self.u32()?))
    }
    pub fn vec3(&mut self) -> io::Result<[f32; 3]> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    /// Texto de tamanho fixo terminado em zero.
    pub fn fixed_str(&mut self, n: usize) -> io::Result<String> {
        Ok(c_str(self.bytes(n)?))
    }

    pub fn chunk(&mut self) -> io::Result<Chunk> {
        let kind = self.u32()?;
        let size = self.u32()? as usize;
        let version = self.u32()?;
        let start = self.pos;
        if start + size > self.data.len() {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("bloco 0x{kind:X} maior que o arquivo"),
            ));
        }
        Ok(Chunk { kind, size, version, start })
    }

    /// Le o proximo bloco e confere o tipo.
    pub fn expect(&mut self, kind: u32) -> io::Result<Chunk> {
        let c = self.chunk()?;
        if c.kind != kind {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                format!("esperava bloco 0x{kind:X}, veio 0x{:X}", c.kind),
            ));
        }
        Ok(c)
    }

    pub fn goto(&mut self, pos: usize) {
        self.pos = pos.min(self.data.len());
    }
}

pub fn c_str(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    b[..end].iter().map(|&c| c as char).collect()
}
