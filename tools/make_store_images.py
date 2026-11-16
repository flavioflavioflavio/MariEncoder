"""Monta as screenshots da página da Microsoft Store (1920x1080) a partir das
capturas do app, com título e subtítulo em cada idioma.

Uso:
    python tools/make_store_images.py <pasta com capturas> <pasta de saída>

A pasta de capturas tem uma subpasta por idioma (pt-BR, en, es) com
1-queue.png, 2-presets.png, 3-compact.png e 4-about.png (janelas do app).
A imagem 0 é uma ilustração do menu de botão direito. Requer Pillow.
"""
import sys
from pathlib import Path

from PIL import Image, ImageDraw, ImageFilter, ImageFont

ROOT = Path(__file__).resolve().parent.parent
W, H = 1920, 1080
ORANGE = (247, 148, 29)
LIGHT = (250, 179, 94)
BG = (18, 18, 21)
TEXT = (236, 236, 241)
MUTED = (160, 160, 172)

FONT_DIR = Path("C:/Windows/Fonts")


def font(size: int, bold: bool = False) -> ImageFont.FreeTypeFont:
    if bold:
        return ImageFont.truetype(str(ROOT / "assets" / "fonts" / "Outfit-Black.ttf"), size)
    return ImageFont.truetype(str(FONT_DIR / "segoeui.ttf"), size)


def semibold(size: int) -> ImageFont.FreeTypeFont:
    return ImageFont.truetype(str(FONT_DIR / "seguisb.ttf"), size)


COPY = {
    "pt-BR": [
        ("Clique com o botão direito. Pronto.", "Escolha um preset no menu do Windows e o vídeo é convertido na hora."),
        ("Vários vídeos, uma fila só", "Arraste arquivos ou pastas, escolha o preset de cada um e acompanhe tudo."),
        ("Crie seus próprios presets", "WhatsApp, 3 Mbps, YouTube, GIF, MP3… ou do jeito que você precisar."),
        ("Rápido e discreto", "Uma janela pequena mostra o progresso e fecha sozinha quando termina."),
        ("Grátis, sem anúncios, código aberto", "Usa os encoders do Windows e da sua placa de vídeo. Nada sai do seu PC."),
    ],
    "en": [
        ("Right-click. Done.", "Pick a preset from the Windows menu and the video is converted right away."),
        ("Many videos, one queue", "Drag files or folders, pick a preset for each one and follow along."),
        ("Make your own presets", "WhatsApp, 3 Mbps, YouTube, GIF, MP3… or exactly what you need."),
        ("Fast and unobtrusive", "A small window shows the progress and closes itself when it's done."),
        ("Free, no ads, open source", "Uses the encoders built into Windows and your graphics card. Nothing leaves your PC."),
    ],
    "es": [
        ("Clic derecho. Listo.", "Elige un preset en el menú de Windows y el vídeo se convierte al instante."),
        ("Muchos vídeos, una sola cola", "Arrastra archivos o carpetas, elige el preset de cada uno y sigue el progreso."),
        ("Crea tus propios presets", "WhatsApp, 3 Mbps, YouTube, GIF, MP3… o lo que necesites."),
        ("Rápido y discreto", "Una ventana pequeña muestra el progreso y se cierra sola al terminar."),
        ("Gratis, sin anuncios, código abierto", "Usa los codificadores de Windows y de tu tarjeta gráfica. Nada sale de tu PC."),
    ],
}

MENU = {
    "pt-BR": (["Abrir", "Reproduzir", "Compartilhar"], ["WhatsApp", "3 Mbps", "MP4", "YouTube", "HEVC compacto", "GIF", "Extrair MP3"], ["Abrir no Mari Encoder…", "Editar presets…"]),
    "en": (["Open", "Play", "Share"], ["WhatsApp", "3 Mbps", "MP4", "YouTube", "Compact HEVC", "GIF", "Extract MP3"], ["Open in Mari Encoder…", "Edit presets…"]),
    "es": (["Abrir", "Reproducir", "Compartir"], ["WhatsApp", "3 Mbps", "MP4", "YouTube", "HEVC compacto", "GIF", "Extraer MP3"], ["Abrir en Mari Encoder…", "Editar presets…"]),
}


def background() -> Image.Image:
    img = Image.new("RGB", (W, H), BG)
    glow = Image.new("RGB", (W, H), BG)
    draw = ImageDraw.Draw(glow)
    draw.ellipse((W - 900, -500, W + 500, 700), fill=(120, 66, 12))
    draw.ellipse((-600, H - 300, 500, H + 600), fill=(60, 34, 10))
    return Image.blend(img, glow.filter(ImageFilter.GaussianBlur(220)), 0.85)


def header(img: Image.Image, title: str, subtitle: str, width: int = 820) -> None:
    draw = ImageDraw.Draw(img)
    logo = Image.open(ROOT / "assets" / "logo-horizontal.png").convert("RGBA")
    logo = logo.resize((round(logo.width * 56 / logo.height), 56), Image.LANCZOS)
    img.paste(logo, (110, 110), logo)
    y = 250
    for line in wrap(title, font(78, True), width):
        draw.text((110, y), line, font=font(78, True), fill=TEXT)
        y += 92
    y += 18
    for line in wrap(subtitle, font(34), width):
        draw.text((110, y), line, font=font(34), fill=MUTED)
        y += 48


def wrap(text: str, f: ImageFont.FreeTypeFont, width: int) -> list[str]:
    words, lines, cur = text.split(), [], ""
    for word in words:
        test = f"{cur} {word}".strip()
        if f.getlength(test) <= width:
            cur = test
        else:
            lines.append(cur)
            cur = word
    if cur:
        lines.append(cur)
    return lines


def shadowed(img: Image.Image, shot: Image.Image, pos: tuple[int, int]) -> None:
    shadow = Image.new("RGBA", (shot.width + 120, shot.height + 120), (0, 0, 0, 0))
    ImageDraw.Draw(shadow).rounded_rectangle((60, 70, shot.width + 60, shot.height + 70), 18, fill=(0, 0, 0, 170))
    shadow = shadow.filter(ImageFilter.GaussianBlur(28))
    img.paste(shadow, (pos[0] - 60, pos[1] - 60), shadow)
    mask = Image.new("L", shot.size, 0)
    ImageDraw.Draw(mask).rounded_rectangle((0, 0, shot.width, shot.height), 12, fill=255)
    img.paste(shot, pos, mask)


def with_window(title: str, subtitle: str, shot_path: Path, max_w: int, max_h: int) -> Image.Image:
    img = background().convert("RGBA")
    header(img, title, subtitle, width=620)
    shot = Image.open(shot_path).convert("RGBA")
    scale = min(max_w / shot.width, max_h / shot.height, 1.0)
    shot = shot.resize((round(shot.width * scale), round(shot.height * scale)), Image.LANCZOS)
    x = W - shot.width - 90
    y = (H - shot.height) // 2 + 20
    shadowed(img, shot, (x, y))
    return img.convert("RGB")


def menu_card(items, highlight=None, separators=(), icon_rows=(), arrow_rows=()):
    row_h, pad, w = 58, 14, 470
    h = pad * 2 + row_h * len(items) + 14 * len(separators)
    card = Image.new("RGBA", (w, h), (0, 0, 0, 0))
    d = ImageDraw.Draw(card)
    d.rounded_rectangle((0, 0, w - 1, h - 1), 14, fill=(44, 44, 50, 255), outline=(70, 70, 80, 255))
    y = pad
    for i, label in enumerate(items):
        if i in separators:
            d.line((18, y + 6, w - 18, y + 6), fill=(80, 80, 90), width=2)
            y += 14
        if i == highlight:
            d.rounded_rectangle((8, y + 3, w - 8, y + row_h - 3), 8, fill=(70, 70, 80))
        x = 30
        if i in icon_rows:
            icon = Image.open(ROOT / "assets" / "icon.png").convert("RGBA").resize((30, 30), Image.LANCZOS)
            card.paste(icon, (x, y + 14), icon)
        x = 80
        d.text((x, y + row_h // 2), label, font=semibold(28) if i == highlight else font(28), fill=TEXT, anchor="lm")
        if i in arrow_rows:
            d.text((w - 34, y + row_h // 2), "›", font=semibold(34), fill=TEXT, anchor="mm")
        y += row_h
    return card


def hero(lang: str) -> Image.Image:
    title, subtitle = COPY[lang][0]
    img = background().convert("RGBA")
    header(img, title, subtitle, width=700)
    top, presets, extra = MENU[lang]
    main = menu_card(["Mari Encoder", *top], highlight=0, icon_rows=(0,), arrow_rows=(0,), separators=(1,))
    sub = menu_card([*presets, *extra], highlight=0, separators=(len(presets),))
    # Arquivo de vídeo clicado.
    d = ImageDraw.Draw(img)
    fx, fy = 860, 200
    d.rounded_rectangle((fx, fy, fx + 120, fy + 150), 12, fill=(52, 52, 60), outline=(90, 90, 100), width=2)
    d.polygon([(fx + 45, fy + 50), (fx + 45, fy + 100), (fx + 88, fy + 75)], fill=ORANGE)
    d.text((fx + 60, fy + 180), "video.mp4", font=font(26), fill=TEXT, anchor="mm")
    mx, my = fx + 40, fy + 220
    shadowed(img, main, (mx, my))
    shadowed(img, sub, (mx + main.width - 14, my))
    return img.convert("RGB")


def main() -> None:
    raw = Path(sys.argv[1])
    out = Path(sys.argv[2])
    for lang in COPY:
        dest = out / lang
        dest.mkdir(parents=True, exist_ok=True)
        hero(lang).save(dest / "1-menu.png", optimize=True)
        shots = [("2-fila.png", "1-queue.png", 1, 1020, 800), ("3-presets.png", "2-presets.png", 2, 1020, 800),
                 ("4-janela-rapida.png", "3-compact.png", 3, 860, 560), ("5-sobre.png", "4-about.png", 4, 1020, 800)]
        for name, src, idx, mw, mh in shots:
            title, subtitle = COPY[lang][idx]
            with_window(title, subtitle, raw / lang / src, mw, mh).save(dest / name, optimize=True)
        print(lang, "ok")


if __name__ == "__main__":
    main()
