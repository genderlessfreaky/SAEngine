//! Leitor de arquivos .img do San Andreas (formato VER2).
//!
//! O .img e um "pacote" com milhares de arquivos dentro (modelos .dff,
//! texturas .txd, colisoes .col...). Cabecalho:
//!   "VER2" + quantidade (u32) + N entradas de 32 bytes:
//!   offset (u32, em setores de 2048 bytes), tamanho (u16), tamanho2 (u16), nome (24 bytes)

use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

const SECTOR: u64 = 2048;

#[derive(Debug, Clone, Copy)]
pub struct ImgEntry {
    pub offset: u64,
    pub size: u64,
}

pub struct ImgArchive {
    pub path: PathBuf,
    /// nome em minusculas -> posicao no arquivo
    pub entries: HashMap<String, ImgEntry>,
}

impl ImgArchive {
    pub fn open(path: &Path) -> io::Result<Self> {
        let mut f = File::open(path)?;
        let mut head = [0u8; 8];
        f.read_exact(&mut head)?;
        if &head[0..4] != b"VER2" {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "arquivo .img nao e do tipo VER2 (San Andreas)",
            ));
        }
        let count = u32::from_le_bytes([head[4], head[5], head[6], head[7]]) as usize;
        let mut dir = vec![0u8; count * 32];
        f.read_exact(&mut dir)?;

        let mut entries = HashMap::with_capacity(count);
        for e in dir.chunks_exact(32) {
            let offset = u32::from_le_bytes([e[0], e[1], e[2], e[3]]) as u64;
            let s1 = u16::from_le_bytes([e[4], e[5]]) as u64;
            let s2 = u16::from_le_bytes([e[6], e[7]]) as u64;
            let size = if s1 != 0 { s1 } else { s2 };
            let name_end = e[8..32].iter().position(|&b| b == 0).unwrap_or(24);
            let name: String = e[8..8 + name_end].iter().map(|&b| b as char).collect();
            entries.insert(
                name.to_lowercase(),
                ImgEntry { offset: offset * SECTOR, size: size * SECTOR },
            );
        }
        Ok(Self { path: path.to_path_buf(), entries })
    }

    pub fn contains(&self, name: &str) -> bool {
        self.entries.contains_key(&name.to_lowercase())
    }

    /// Le um arquivo de dentro do .img pelo nome (ex: "infernus.dff").
    pub fn read(&self, name: &str) -> io::Result<Vec<u8>> {
        let entry = self.entries.get(&name.to_lowercase()).ok_or_else(|| {
            io::Error::new(io::ErrorKind::NotFound, format!("'{name}' nao existe no {}", self.path.display()))
        })?;
        let mut f = File::open(&self.path)?;
        f.seek(SeekFrom::Start(entry.offset))?;
        let mut buf = vec![0u8; entry.size as usize];
        f.read_exact(&mut buf)?;
        Ok(buf)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Monta um .img VER2 minusculo na memoria e confere a leitura.
    #[test]
    fn le_arquivo_pelo_nome_sem_diferenciar_maiusculas() {
        let conteudo = b"ola san andreas";
        let mut entrada = [0u8; 32];
        entrada[0..4].copy_from_slice(&1u32.to_le_bytes()); // comeca no setor 1
        entrada[4..6].copy_from_slice(&1u16.to_le_bytes()); // 1 setor
        entrada[8..8 + 7].copy_from_slice(b"Teste.d");

        let mut dados = Vec::new();
        dados.extend_from_slice(b"VER2");
        dados.extend_from_slice(&1u32.to_le_bytes());
        dados.extend_from_slice(&entrada);
        dados.resize(2048, 0);
        dados.extend_from_slice(conteudo);
        dados.resize(4096, 0);

        let caminho = std::env::temp_dir().join("sae_rw_teste.img");
        std::fs::write(&caminho, &dados).unwrap();

        let img = ImgArchive::open(&caminho).unwrap();
        assert!(img.contains("TESTE.D"));
        let lido = img.read("teste.d").unwrap();
        assert_eq!(lido.len(), 2048);
        assert_eq!(&lido[..conteudo.len()], conteudo);
        assert!(img.read("nao_existe.dff").is_err());
        let _ = std::fs::remove_file(caminho);
    }

    #[test]
    fn recusa_arquivo_que_nao_e_ver2() {
        let caminho = std::env::temp_dir().join("sae_rw_ruim.img");
        std::fs::write(&caminho, b"XXXX\0\0\0\0").unwrap();
        assert!(ImgArchive::open(&caminho).is_err());
        let _ = std::fs::remove_file(caminho);
    }
}
