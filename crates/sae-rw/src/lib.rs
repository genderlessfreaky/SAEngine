//! sae-rw: leitores dos formatos de arquivo do GTA San Andreas (RenderWare).
//! Codigo proprio da SAEngine, escrito do zero (a unica dependencia e a `log`).

pub mod dff;
pub mod img;
pub mod stream;
pub mod txd;

#[cfg(test)]
mod tests {
    use super::*;

    /// Teste com arquivos reais. Rode com:
    ///   SAE_TEST_DIR=pasta_com_arquivos cargo test -- --nocapture
    #[test]
    fn arquivos_reais() {
        let Ok(dir) = std::env::var("SAE_TEST_DIR") else { return };
        let dir = std::path::PathBuf::from(dir);
        for name in ["infernus.dff", "sw_block01.dff", "wheels.DFF"] {
            let data = std::fs::read(dir.join(name)).unwrap();
            let c = dff::parse_dff(&data).unwrap();
            println!("== {name}: {} frames, {} geometrias, {} atomics", c.frames.len(), c.geometries.len(), c.atomics.len());
            for (i, f) in c.frames.iter().enumerate() {
                println!("  frame {i:2} '{}' pai={} pos={:?}", f.name, f.parent, f.pos);
            }
            for a in &c.atomics {
                let g = &c.geometries[a.geometry];
                let texs: Vec<_> = g.materials.iter().map(|m| (m.texture.clone().unwrap_or_default(), m.color)).collect();
                println!("  atomic frame='{}' verts={} tris={} uvs={} normals={} mats={:?}",
                    c.frames[a.frame].name, g.positions.len(), g.triangles.len(), g.uvs.len(), g.normals.len(), texs);
            }
        }
        for name in ["infernus.txd", "vehicle.txd"] {
            let data = std::fs::read(dir.join(name)).unwrap();
            let t = txd::parse_txd(&data).unwrap();
            println!("== {name}: {} texturas", t.len());
            for tex in &t {
                println!("  '{}' {}x{} alpha={}", tex.name, tex.width, tex.height, tex.has_alpha);
                let out = dir.join(format!("{}.rgba_{}x{}", tex.name, tex.width, tex.height));
                std::fs::write(out, &tex.rgba).unwrap();
            }
        }
    }
}
