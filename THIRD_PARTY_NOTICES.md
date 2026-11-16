# Componentes de terceiros — Mari Encoder

## FFmpeg

O Mari Encoder acompanha o FFmpeg como **programa separado** (`ffmpeg\ffmpeg.exe`,
`ffmpeg\ffprobe.exe` e as DLLs ao lado), chamado por linha de comando.

- **Licença:** GNU Lesser General Public License, versão 3 (LGPL v3). O texto
  está em `ffmpeg\LICENSE.txt`, dentro do pacote.
- **Build:** FFmpeg 9.0.2 (`n9.0.2-57-gd838f9f9ae`), variante `lgpl-shared` do
  projeto [BtbN/FFmpeg-Builds](https://github.com/BtbN/FFmpeg-Builds), release
  [`autobuild-2026-11-15-12-40`](https://github.com/BtbN/FFmpeg-Builds/releases/tag/autobuild-2026-11-15-12-40).
  Essa variante não inclui componentes GPL (como x264 e x265).
- **Código-fonte:** <https://git.ffmpeg.org/ffmpeg.git>, commit `d838f9f9ae`;
  os scripts de build e as bibliotecas usadas estão no repositório do BtbN.
- Você pode trocar o FFmpeg por outra versão compatível: em **Ajustes → ffmpeg**,
  aponte para a pasta do seu `ffmpeg.exe` (na versão do GitHub).

## Codecs

H.264, HEVC e AAC são gerados pelos encoders do próprio Windows (Media
Foundation) ou da placa de vídeo (NVIDIA NVENC, Intel Quick Sync, AMD AMF). O
Mari Encoder não distribui encoders próprios desses formatos. AV1 (SVT-AV1),
VP9 (libvpx), Opus e MP3 (LAME) vêm do FFmpeg.

## Fonte da marca

Logo desenhado com a fonte **Outfit** (SIL Open Font License 1.1), usada apenas
para gerar as imagens; a fonte não é distribuída no app.

## Crates Rust

O app usa bibliotecas Rust de código aberto (eframe/egui, serde, windows-rs,
winreg, rfd e dependências), sob as licenças MIT e/ou Apache-2.0. A lista
completa está no `Cargo.lock` do repositório.

---

# Third-party notices — Mari Encoder

- **FFmpeg** 9.0.2 (`lgpl-shared` build by BtbN/FFmpeg-Builds, release
  `autobuild-2026-11-15-12-40`), shipped as a separate program under the
  **LGPL v3**. License text in `ffmpeg\LICENSE.txt`; source at
  <https://git.ffmpeg.org/ffmpeg.git>, commit `d838f9f9ae`. No GPL components
  (x264/x265) are included.
- **Codecs:** H.264, HEVC and AAC are encoded by Windows (Media Foundation) or the
  GPU vendor's encoder; Mari Encoder ships no encoder of its own for them.
- **Outfit** font (SIL OFL 1.1), used only to render the logo images.
- **Rust crates** under MIT and/or Apache-2.0; see `Cargo.lock`.
