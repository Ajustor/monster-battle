#!/usr/bin/env python3
"""
Génère les icônes Android pour Monster Battle à partir du sprite pixel-art
du Dragon (Feu), mascotte du jeu.

Produit des PNG aux tailles standard Android :
  - mdpi     48×48
  - hdpi     72×72
  - xhdpi    96×96
  - xxhdpi  144×144
  - xxxhdpi 192×192
  - Play Store 512×512

Usage : python3 scripts/gen_icon.py
"""

from pathlib import Path

try:
    from PIL import Image, ImageDraw
except ImportError:
    print("Pillow requis : pip install Pillow")
    raise SystemExit(1)

# ── Sprite 16×16 du Dragon (Feu) — même grille que sprites.rs ────
DRAGON_GRID = [
    "................",
    "....A......A....",
    "....AMMMMMMA....",
    ".....MWMMWM.....",
    "....AMMMMMM.....",
    ".....MMDDMM.....",
    "...DD.MMMM.DD...",
    "...DDDMAAMDDD...",
    "...DDDMAAMDDDA..",
    "......MMMM.MM...",
    "......M..M......",
    "......M..M......",
    ".....DD..DD.....",
    "................",
    "................",
    "................",
]

# ── Palette Feu ──────────────────────────────────────────────────
PALETTE = {
    "M": (244, 67, 54, 255),     # Main — Rouge
    "D": (198, 40, 40, 255),     # Dark — Rouge foncé
    "A": (255, 138, 101, 255),   # Accent — Orange clair
    "W": (255, 255, 255, 255),   # Blanc (yeux)
    "X": (121, 85, 72, 255),     # Brun
    ".": (0, 0, 0, 0),           # Transparent
}

# ── Couleur de fond ──────────────────────────────────────────────
BG_COLOR_CENTER = (30, 30, 46, 255)      # Fond sombre bleuté
BG_COLOR_EDGE   = (17, 17, 27, 255)      # Bord encore plus sombre

# ── Tailles de sortie ───────────────────────────────────────────
SIZES = {
    "mipmap-mdpi":    48,
    "mipmap-hdpi":    72,
    "mipmap-xhdpi":   96,
    "mipmap-xxhdpi":  144,
    "mipmap-xxxhdpi": 192,
}

PLAYSTORE_SIZE = 512


def render_sprite(size: int) -> Image.Image:
    """Rend le sprite pixel-art sur un fond avec coins arrondis."""
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    # Fond arrondi
    radius = max(size // 6, 4)
    draw.rounded_rectangle(
        [(0, 0), (size - 1, size - 1)],
        radius=radius,
        fill=BG_COLOR_CENTER,
    )

    # Dessiner le sprite centré avec marge
    grid_size = 16
    margin = max(size // 8, 2)
    sprite_area = size - 2 * margin
    pixel_w = sprite_area / grid_size

    for row_idx, row in enumerate(DRAGON_GRID):
        for col_idx, ch in enumerate(row):
            if ch == ".":
                continue
            color = PALETTE.get(ch, PALETTE["."])
            x0 = int(margin + col_idx * pixel_w)
            y0 = int(margin + row_idx * pixel_w)
            x1 = int(margin + (col_idx + 1) * pixel_w)
            y1 = int(margin + (row_idx + 1) * pixel_w)
            draw.rectangle([x0, y0, x1, y1], fill=color)

    return img


def main():
    project_root = Path(__file__).resolve().parent.parent
    res_dir = project_root / "crates" / "android" / "res"

    # Icônes Android par densité
    for folder, size in SIZES.items():
        out_dir = res_dir / folder
        out_dir.mkdir(parents=True, exist_ok=True)
        icon = render_sprite(size)
        out_path = out_dir / "ic_launcher.png"
        icon.save(out_path, "PNG")
        print(f"  ✅ {out_path.relative_to(project_root)}  ({size}×{size})")

    # Icône Play Store
    store_dir = project_root / "crates" / "android" / "metadata" / "playstore"
    store_dir.mkdir(parents=True, exist_ok=True)
    icon = render_sprite(PLAYSTORE_SIZE)
    out_path = store_dir / "ic_launcher.png"
    icon.save(out_path, "PNG")
    print(f"  ✅ {out_path.relative_to(project_root)}  ({PLAYSTORE_SIZE}×{PLAYSTORE_SIZE})")

    print(f"\n🎉 Icônes générées dans {res_dir.relative_to(project_root)}/")


if __name__ == "__main__":
    main()
