"""Monta a pasta de importação da página da Microsoft Store (pt-BR, en-US, es-ES).

Uso:
    python tools/store_listing.py <CSV exportado do Partner Center> <pasta de saída>

Lê o CSV de "Exportar listagens", preenche textos, screenshots e logos nas
colunas de cada idioma e grava <saída>/listingData.csv com as imagens ao lado
(caminhos relativos). Depois é só usar "Importar listagens" com essa pasta.
As screenshots vêm de dist/store/<idioma> (tools/make_store_images.py).
"""
import csv
import shutil
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter

ROOT = Path(__file__).resolve().parent.parent

LANGS = {"pt-br": "pt-BR", "en-us": "en", "es-es": "es"}

TEXT = {
    "pt-br": {
        "Title": "Mari Encoder",
        "ShortDescription": "Converta vídeos com um clique pelo botão direito do Windows: WhatsApp, MP4, YouTube, GIF, MP3 e presets seus.",
        "Description": """Mari Encoder converte vídeos em um clique, direto do botão direito do Windows.

Clique com o botão direito em um vídeo, escolha um preset e pronto: o arquivo convertido aparece ao lado do original, sem abrir programa nenhum. Uma janela pequena mostra o progresso e fecha sozinha quando termina.

PRESETS PRONTOS
• WhatsApp — vídeo leve em 720p, pronto para enviar
• 3 Mbps — tamanho previsível para upload
• MP4 — compatível com qualquer aparelho
• YouTube — alta qualidade para publicar
• HEVC compacto — metade do tamanho, mesma qualidade (requer placa de vídeo compatível)
• GIF — animação leve para compartilhar
• Extrair MP3 — só o áudio

CRIE OS SEUS
Escolha formato (MP4, MOV, MKV, WebM, GIF, MP3, M4A, WAV), codec (H.264, HEVC, AV1, VP9, ProRes), qualidade, bitrate fixo ou tamanho-alvo do arquivo, resolução e FPS máximos, áudio e pasta de saída. Cada preset aparece no menu de botão direito.

VÁRIOS VÍDEOS DE UMA VEZ
Selecione vários arquivos ou arraste pastas inteiras: tudo entra numa fila só. Na janela completa você troca o preset de cada arquivo, cancela, tenta de novo e abre a pasta do resultado.

RÁPIDO
Usa os encoders do próprio Windows e da sua placa de vídeo (NVIDIA, Intel, AMD) para converter muito mais rápido que o processador sozinho.

PRIVADO E GRATUITO
Tudo acontece no seu computador: nenhum vídeo é enviado para a internet, sem conta, sem anúncios, sem versão paga. Código aberto (GPLv3) no GitHub.

Em português, inglês e espanhol.

Dedicado a Mariana Lima, com muito amor.""",
        "Features": [
            "Converta vídeos pelo botão direito do Windows, com um item por preset",
            "Presets prontos: WhatsApp, 3 Mbps, MP4, YouTube, HEVC, GIF e MP3",
            "Crie presets com formato, codec, qualidade, bitrate ou tamanho-alvo",
            "Fila única para vários arquivos e pastas inteiras",
            "Aceleração pela placa de vídeo (NVIDIA, Intel e AMD)",
            "Janela rápida que mostra o progresso e fecha sozinha",
            "Nada sai do seu PC: sem conta, sem anúncios, sem coleta de dados",
            "Gratuito e de código aberto",
        ],
        "Search": ["converter vídeo", "compressor de vídeo", "whatsapp", "mp4", "gif", "mp3", "ffmpeg"],
        "Captions": [
            "Clique com o botão direito e escolha o preset",
            "Vários vídeos numa fila só",
            "Crie seus próprios presets",
            "Janela rápida com o progresso",
            "Grátis, sem anúncios, código aberto",
        ],
        "Copyright": "© 2026 Flávio Takemoto",
        "License": "Software livre sob a licença GPL-3.0-or-later. Código-fonte: https://github.com/flavioflavioflavio/MariEncoder. O FFmpeg incluído é distribuído sob a LGPL v3.",
    },
    "en-us": {
        "Title": "Mari Encoder",
        "ShortDescription": "Convert videos with one click from the Windows right-click menu: WhatsApp, MP4, YouTube, GIF, MP3 and your own presets.",
        "Description": """Mari Encoder converts videos with one click, straight from the Windows right-click menu.

Right-click a video, pick a preset and you're done: the converted file shows up next to the original, without opening any program. A small window shows the progress and closes itself when it's finished.

READY-MADE PRESETS
• WhatsApp — light 720p video, ready to send
• 3 Mbps — predictable size for uploads
• MP4 — plays on any device
• YouTube — high quality for publishing
• Compact HEVC — half the size, same quality (requires a compatible graphics card)
• GIF — lightweight animation to share
• Extract MP3 — audio only

MAKE YOUR OWN
Choose the format (MP4, MOV, MKV, WebM, GIF, MP3, M4A, WAV), codec (H.264, HEVC, AV1, VP9, ProRes), quality, fixed bitrate or target file size, maximum resolution and FPS, audio and output folder. Every preset shows up in the right-click menu.

MANY VIDEOS AT ONCE
Select several files or drag whole folders: everything goes into a single queue. In the full window you can change each file's preset, cancel, retry and open the output folder.

FAST
Uses the encoders built into Windows and your graphics card (NVIDIA, Intel, AMD) to convert much faster than the CPU alone.

PRIVATE AND FREE
Everything happens on your computer: no video is uploaded, no account, no ads, no paid version. Open source (GPLv3) on GitHub.

Available in English, Portuguese and Spanish.

Dedicated to Mariana Lima, with all my love.""",
        "Features": [
            "Convert videos from the Windows right-click menu, one item per preset",
            "Ready-made presets: WhatsApp, 3 Mbps, MP4, YouTube, HEVC, GIF and MP3",
            "Make presets with format, codec, quality, bitrate or target file size",
            "One queue for many files and whole folders",
            "Graphics card acceleration (NVIDIA, Intel and AMD)",
            "Quick window that shows the progress and closes itself",
            "Nothing leaves your PC: no account, no ads, no data collection",
            "Free and open source",
        ],
        "Search": ["video converter", "video compressor", "whatsapp", "mp4", "gif", "mp3", "ffmpeg"],
        "Captions": [
            "Right-click and pick a preset",
            "Many videos in one queue",
            "Make your own presets",
            "Quick window with the progress",
            "Free, no ads, open source",
        ],
        "Copyright": "© 2026 Flávio Takemoto",
        "License": "Free software under the GPL-3.0-or-later license. Source code: https://github.com/flavioflavioflavio/MariEncoder. The bundled FFmpeg is distributed under the LGPL v3.",
    },
    "es-es": {
        "Title": "Mari Encoder",
        "ShortDescription": "Convierte vídeos con un clic desde el clic derecho de Windows: WhatsApp, MP4, YouTube, GIF, MP3 y tus propios presets.",
        "Description": """Mari Encoder convierte vídeos con un clic, directamente desde el clic derecho de Windows.

Haz clic derecho en un vídeo, elige un preset y listo: el archivo convertido aparece junto al original, sin abrir ningún programa. Una ventana pequeña muestra el progreso y se cierra sola al terminar.

PRESETS LISTOS
• WhatsApp — vídeo ligero en 720p, listo para enviar
• 3 Mbps — tamaño previsible para subir
• MP4 — compatible con cualquier dispositivo
• YouTube — alta calidad para publicar
• HEVC compacto — la mitad del tamaño, la misma calidad (requiere una tarjeta gráfica compatible)
• GIF — animación ligera para compartir
• Extraer MP3 — solo el audio

CREA LOS TUYOS
Elige el formato (MP4, MOV, MKV, WebM, GIF, MP3, M4A, WAV), el códec (H.264, HEVC, AV1, VP9, ProRes), la calidad, bitrate fijo o tamaño de archivo objetivo, resolución y FPS máximos, el audio y la carpeta de salida. Cada preset aparece en el menú del clic derecho.

MUCHOS VÍDEOS A LA VEZ
Selecciona varios archivos o arrastra carpetas enteras: todo entra en una sola cola. En la ventana completa puedes cambiar el preset de cada archivo, cancelar, reintentar y abrir la carpeta del resultado.

RÁPIDO
Usa los codificadores de Windows y de tu tarjeta gráfica (NVIDIA, Intel, AMD) para convertir mucho más rápido que solo con el procesador.

PRIVADO Y GRATUITO
Todo ocurre en tu ordenador: ningún vídeo se sube a internet, sin cuenta, sin anuncios, sin versión de pago. Código abierto (GPLv3) en GitHub.

Disponible en español, inglés y portugués.

Dedicado a Mariana Lima, con mucho amor.""",
        "Features": [
            "Convierte vídeos desde el clic derecho de Windows, un elemento por preset",
            "Presets listos: WhatsApp, 3 Mbps, MP4, YouTube, HEVC, GIF y MP3",
            "Crea presets con formato, códec, calidad, bitrate o tamaño objetivo",
            "Una sola cola para muchos archivos y carpetas enteras",
            "Aceleración por tarjeta gráfica (NVIDIA, Intel y AMD)",
            "Ventana rápida que muestra el progreso y se cierra sola",
            "Nada sale de tu PC: sin cuenta, sin anuncios, sin recopilar datos",
            "Gratuito y de código abierto",
        ],
        "Search": ["convertir vídeo", "comprimir vídeo", "whatsapp", "mp4", "gif", "mp3", "ffmpeg"],
        "Captions": [
            "Clic derecho y elige el preset",
            "Muchos vídeos en una sola cola",
            "Crea tus propios presets",
            "Ventana rápida con el progreso",
            "Gratis, sin anuncios, código abierto",
        ],
        "Copyright": "© 2026 Flávio Takemoto",
        "License": "Software libre bajo la licencia GPL-3.0-or-later. Código fuente: https://github.com/flavioflavioflavio/MariEncoder. El FFmpeg incluido se distribuye bajo la LGPL v3.",
    },
}

SHOTS = ["1-menu.png", "2-fila.png", "3-presets.png", "4-janela-rapida.png", "5-sobre.png"]


def logo(size: tuple[int, int]) -> Image.Image:
    """Ícone no fundo escuro da marca, com o brilho laranja das screenshots."""
    w, h = size
    img = Image.new("RGB", size, (18, 18, 21))
    glow = Image.new("RGB", size, (18, 18, 21))
    ImageDraw.Draw(glow).ellipse((w * 0.1, h * 0.1, w * 0.9, h * 0.9), fill=(110, 60, 12))
    img = Image.blend(img, glow.filter(ImageFilter.GaussianBlur(min(w, h) // 5)), 0.7)
    icon = Image.open(ROOT / "assets" / "icon.png").convert("RGBA")
    side = round(min(w, h) * 0.56)
    icon = icon.resize((side, side), Image.LANCZOS)
    img.paste(icon, ((w - side) // 2, (h - side) // 2), icon)
    return img


def main() -> None:
    template, out = Path(sys.argv[1]), Path(sys.argv[2])
    if out.exists():
        shutil.rmtree(out)
    out.mkdir(parents=True)

    # Logos da Store (iguais nos três idiomas).
    logos = {"StoreLogo300x300": (300, 300), "StoreLogo1080x1080": (1080, 1080), "StoreLogo720x1080": (720, 1080)}
    for name, size in logos.items():
        logo(size).save(out / f"{name}.png", optimize=True)

    # Tudo na mesma pasta do CSV: o Partner Center não aceitou caminhos com subpasta.
    for column, folder in LANGS.items():
        for name in SHOTS:
            shutil.copy(ROOT / "dist" / "store" / folder / name, out / f"{folder}_{name}")

    with open(template, encoding="utf-8-sig", newline="") as f:
        rows = list(csv.DictReader(f))
        fields = list(rows[0].keys())

    for row in rows:
        field = row["Field"]
        for column, folder in LANGS.items():
            t = TEXT[column]
            value = None
            if field in ("Title", "ShortDescription", "Description"):
                value = t[field]
            elif field == "DevStudio":
                value = "Flávio Takemoto"
            elif field == "CopyrightTrademarkInformation":
                value = t["Copyright"]
            elif field == "AdditionalLicenseTerms":
                value = t["License"]
            elif field.startswith("Feature") and field[7:].isdigit():
                i = int(field[7:]) - 1
                value = t["Features"][i] if i < len(t["Features"]) else ""
            elif field.startswith("SearchTerm"):
                i = int(field[10:]) - 1
                value = t["Search"][i] if i < len(t["Search"]) else ""
            elif field.startswith("DesktopScreenshotCaption"):
                i = int(field[24:]) - 1
                value = t["Captions"][i] if i < len(t["Captions"]) else ""
            elif field.startswith("DesktopScreenshot"):
                i = int(field[17:]) - 1
                value = f"{folder}_{SHOTS[i]}" if i < len(SHOTS) else ""
            elif field in logos:
                value = f"{field}.png"
            if value is not None:
                row[column] = value

    with open(out / "listingData.csv", "w", encoding="utf-8-sig", newline="") as f:
        writer = csv.DictWriter(f, fieldnames=fields)
        writer.writeheader()
        writer.writerows(rows)

    # Conferências de limite do Partner Center.
    for column, t in TEXT.items():
        assert len(t["Description"]) <= 10000
        assert all(len(x) <= 200 for x in t["Features"]), column
        assert all(len(x) <= 30 for x in t["Search"]), column
        assert len(t["Search"]) <= 7
    print("pasta de importação pronta em", out)


if __name__ == "__main__":
    main()
