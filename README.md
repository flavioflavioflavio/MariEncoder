# Mari Encoder

Encode de vídeo em um clique, direto do botão direito do Windows.

Clique com o botão direito num vídeo → **Mari Encoder** → escolha um preset
(WhatsApp, 3 Mbps, MP4, YouTube, HEVC, GIF, Extrair MP3 ou os seus). Vários
arquivos selecionados entram numa fila só. A janela completa permite arrastar
pastas, trocar o preset de cada arquivo e criar presets novos.

*Dedicado a Mariana Lima, com muito amor.*

## Codecs

H.264, HEVC e AAC usam os encoders do próprio Windows (Media Foundation) ou da
placa de vídeo (NVIDIA, Intel, AMD). AV1, VP9, Opus e MP3 vêm do FFmpeg (build
LGPL, sem x264/x265). Detalhes em [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Compilar

Requer Rust 1.92+ e, para o pacote da Store, o Windows SDK e Python com Pillow.

```bash
cargo build --release
```

Gera `target/release/MariEncoder.exe` e `MariEncoderMenu.dll` (menu do Windows 11).
O FFmpeg é procurado na pasta do app, em `ffmpeg\` ao lado dele e no PATH.

### Pacote da Microsoft Store

```bash
pwsh tools/fetch_ffmpeg.ps1
pwsh tools/package_msix.ps1 -Register
pwsh tools/package_msix.ps1 -IdentityName <identidade do Partner Center>
```

O primeiro baixa o FFmpeg LGPL fixado por SHA-256, o segundo registra o pacote
para teste local (Modo de Desenvolvedor) e o terceiro gera o `.msix` em `dist/`.

## Licença

GPL-3.0-or-later — © 2026 Flávio Takemoto. Veja [LICENSE](LICENSE) e
[PRIVACY.md](PRIVACY.md).
