# SAEngine

**Português** | [English](#english)

Engine aberta, em Rust, para qualquer pessoa criar jogos completos com o mundo de San Andreas. Sem escrever código.

Status: **início do projeto.** Hoje existe só a primeira peça, a leitura dos arquivos do jogo. O editor, as peças de missão e o tutorial ainda vão ser construídos.

Leia a visão completa em [VISAO.md](VISAO.md) ou no [site do projeto](docs/index.html).

## O que já funciona

O crate [`sae-rw`](crates/sae-rw) lê os formatos do San Andreas, com código próprio e sem bibliotecas de terceiros para isso:

- `.img` (pacotes do jogo, formato VER2), como o `gta3.img`;
- `.dff` (modelos 3D): peças com nome, geometrias, materiais e texturas usadas;
- `.txd` (texturas): DXT1, DXT3, DXT5, cores de 16 e 32 bits e paleta, convertidas para RGBA.

Testes:

```sh
cargo test
```

Para testar com arquivos reais do jogo, coloque alguns arquivos (por exemplo `infernus.dff`, `infernus.txd`, `vehicle.txd`) numa pasta fora do repositório:

```sh
SAE_TEST_DIR=/caminho/da/pasta cargo test -- --nocapture
```

## Regras importantes

- A SAEngine não contém nem distribui nenhum arquivo do GTA San Andreas. Ela lê a cópia que você já tem instalada.
- É preciso ter o GTA San Andreas original de PC (versão clássica de 2004). A Definitive Edition não é compatível.
- Projeto de fã, sem ligação com a Rockstar Games ou a Take-Two Interactive.

## Como ajudar

O projeto está no começo e a estrutura ainda vai mudar bastante. Abra uma issue para conversar sobre ideias antes de mandar código grande.

## Licença

[GPL v3](LICENSE). Quem modificar a engine e distribuir precisa abrir o código das modificações sob a mesma licença.

---

## English

Open source engine, written in Rust, that lets anyone build complete games using the world of San Andreas. No coding required.

Status: **very early.** Only the first piece exists today: reading the game's files. The editor, the mission pieces and the tutorial are still to be built.

Read the full vision in [VISAO.md](VISAO.md) (Portuguese for now) or on the [project page](docs/index.html).

### What works today

The [`sae-rw`](crates/sae-rw) crate reads San Andreas formats with our own code:

- `.img` archives (VER2), such as `gta3.img`;
- `.dff` 3D models: named parts, geometry, materials and texture names;
- `.txd` textures: DXT1, DXT3, DXT5, 16 and 32 bit colors and palettes, converted to RGBA.

```sh
cargo test
SAE_TEST_DIR=/path/to/some/game/files cargo test -- --nocapture
```

### Important rules

- SAEngine does not contain or distribute any GTA San Andreas file. It reads the copy you already own.
- You need the original PC release of GTA San Andreas (the 2004 version). The Definitive Edition is not supported.
- Fan project, not affiliated with Rockstar Games or Take-Two Interactive.

### License

[GPL v3](LICENSE). If you modify the engine and distribute it, you must release your changes under the same license.
