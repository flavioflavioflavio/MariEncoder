"""Gera a marca do Mari Encoder (família Mari).

O "M" é o mesmo desenho do MariPDF; cada app da família tem a sua cor. Aqui é o
laranja #F7941D da paleta Mari, com a faixa clara por cima.

Uso:
    python tools/make_brand.py

Saída:
    assets/brand/mari-encoder-icon.svg   ícone vetorial
    assets/icon.png (512) / icon.ico   ícone do app e do .exe
    assets/logo-horizontal.png         "MariEncoder" com o M do ícone (Sobre e Store)

Requer: Pillow e resvg-py. Fonte: assets/fonts/Outfit-Black.ttf (OFL).
"""
import io
from pathlib import Path

import resvg_py
from PIL import Image, ImageDraw, ImageFont

ROOT = Path(__file__).resolve().parent.parent
ASSETS = ROOT / "assets"

BASE = "#F7941D"   # laranja Mari
LIGHT = "#FAB35E"  # faixa clara (base misturada com branco)

# Mesmo desenho do M do MariPDF (assets/brand/maripdf-icon.svg), só com as cores trocadas.
ICON_SVG = f"""<svg xmlns="http://www.w3.org/2000/svg" viewBox="41 62 711 676">
  <path fill="{BASE}" d="M51.27,62.56h182.19c3.44,0,6.63,1.78,8.44,4.71l149.66,242.87c3.87,6.28,13.01,6.28,16.88,0l152.57-247.58h197.64v664.96c0,5.47-4.44,9.91-9.91,9.91h-166.28c-5.46,0-9.89-4.42-9.91-9.88l-1.32-352.81c-.04-9.94-13.05-13.65-18.32-5.22l-144.78,231.51c-3.81,6.1-12.64,6.23-16.64.26l-148.8-222.38c-5.45-8.14-18.15-4.29-18.15,5.51v343.09c0,5.47-4.44,9.91-9.91,9.91H51.27c-5.47,0-9.91-4.44-9.91-9.91V72.48c0-5.47,4.44-9.91,9.91-9.91Z"/>
  <path fill="{LIGHT}" d="M41.35,727.52c0,5.47,4.44,9.91,9.91,9.91h163.35c5.47,0,9.91-4.44,9.91-9.91v-180.92c-59.52,49.15-120.89,93.09-183.18,131.68v49.23Z"/>
  <path fill="{LIGHT}" d="M306.56,474.39l84.92,126.91c4,5.98,12.83,5.84,16.64-.26l144.78-231.51c5.27-8.43,18.28-4.72,18.32,5.22l.72,192.26c104.43-143.93,168.32-317.2,175.66-504.44h-125.37c-73.76,143.16-180.23,284.72-315.66,411.83Z"/>
</svg>
"""


def render_mark(height: int) -> Image.Image:
    """M recortado (sem margem), com a altura pedida."""
    width = round(height * 711 / 676)
    png = resvg_py.svg_to_bytes(svg_string=ICON_SVG, width=width, height=height)
    return Image.open(io.BytesIO(bytes(png))).convert("RGBA")


def square_icon(size: int, fill: float = 0.86) -> Image.Image:
    mark = render_mark(round(size * fill))
    canvas = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    canvas.paste(mark, ((size - mark.width) // 2, (size - mark.height) // 2), mark)
    return canvas


def wordmark(height: int = 240) -> Image.Image:
    """M do ícone + "ari" (claro) + "Encoder" (laranja), como o logo do MariPDF."""
    font = ImageFont.truetype(str(ASSETS / "fonts" / "Outfit-Black.ttf"), round(height * 1.34))
    mark = render_mark(height)
    parts = [("ari", LIGHT), ("Encoder", BASE)]
    probe = ImageDraw.Draw(Image.new("RGBA", (1, 1)))
    # Alinha a linha de base do texto com a base do M.
    _, top, _, bottom = probe.textbbox((0, 0), "ari", font=font, anchor="ls")
    gap = round(height * 0.04)
    widths = [probe.textlength(text, font=font) for text, _ in parts]
    width = mark.width + gap + round(sum(widths)) + 8
    canvas = Image.new("RGBA", (width, height + round(height * 0.05)), (0, 0, 0, 0))
    canvas.paste(mark, (0, 0), mark)
    draw = ImageDraw.Draw(canvas)
    x = mark.width + gap
    for (text, color), w in zip(parts, widths):
        draw.text((x, height), text, font=font, fill=color, anchor="ls")
        x += w
    return canvas.crop(canvas.getbbox())


def main() -> None:
    (ASSETS / "brand").mkdir(parents=True, exist_ok=True)
    (ASSETS / "brand" / "mari-encoder-icon.svg").write_text(ICON_SVG, encoding="utf-8")

    big = square_icon(512)
    big.save(ASSETS / "icon.png", optimize=True)
    # O .ico guarda cada tamanho renderizado à parte (fica nítido em 16 e 24 px).
    sizes = [16, 24, 32, 48, 64, 128, 256]
    frames = [square_icon(s, 0.92 if s <= 32 else 0.86) for s in sizes]
    frames[-1].save(ASSETS / "icon.ico", format="ICO", sizes=[(s, s) for s in sizes], append_images=frames[:-1])

    wordmark().save(ASSETS / "logo-horizontal.png", optimize=True)
    print("marca gerada em", ASSETS)


if __name__ == "__main__":
    main()
