#!/usr/bin/env python3
"""Generate icons for aurora_services_demo application."""

from PIL import Image, ImageDraw
import math
import os

SIZES = [86, 108, 128, 172]
OUTPUT_DIR = "aurora_services_demo/rpm/icons"
ICON_NAME = "com.example.aurora_services_demo"


def draw_gear(draw, cx, cy, radius, color):
    inner_radius = radius * 0.4

    draw.ellipse([cx - radius, cy - radius, cx + radius, cy + radius], fill=color)

    teeth = 8
    tooth_radius = radius * 0.25
    gear_outer = radius * 0.85
    for i in range(teeth):
        angle = i * (360 / teeth) * math.pi / 180
        x = cx + gear_outer * math.cos(angle)
        y = cy + gear_outer * math.sin(angle)
        draw.ellipse(
            [x - tooth_radius, y - tooth_radius, x + tooth_radius, y + tooth_radius],
            fill=color,
        )

    draw.ellipse(
        [cx - inner_radius, cy - inner_radius, cx + inner_radius, cy + inner_radius],
        fill=(255, 255, 255),
    )


def draw_notification_bell(draw, cx, cy, size, color):
    bell_width = size * 0.5
    bell_height = size * 0.6

    draw.polygon(
        [
            (cx, cy - bell_height // 2),
            (cx - bell_width // 2, cy + bell_height // 4),
            (cx + bell_width // 2, cy + bell_height // 4),
        ],
        fill=color,
    )

    draw.ellipse(
        [
            cx - bell_width // 2 - 2,
            cy + bell_height // 4 - 4,
            cx + bell_width // 2 + 2,
            cy + bell_height // 4 + 8,
        ],
        fill=color,
    )

    draw.ellipse(
        [
            cx - bell_width // 4,
            cy - bell_height // 2 - 2,
            cx + bell_width // 4,
            cy - bell_height // 2 + 6,
        ],
        fill=color,
    )

    clapper_radius = size * 0.08
    draw.ellipse(
        [
            cx - clapper_radius,
            cy + bell_height // 4 + 4,
            cx + clapper_radius,
            cy + bell_height // 4 + 4 + clapper_radius * 2,
        ],
        fill=color,
    )


def draw_layers(draw, cx, cy, size, color):
    offset = size * 0.12
    rect_width = size * 0.6
    rect_height = size * 0.5

    for i in range(3):
        shade = (
            max(0, color[0] - i * 40),
            max(0, color[1] - i * 40),
            max(0, color[2] - i * 40),
        )
        x_off = i * offset
        y_off = i * offset

        draw.rounded_rectangle(
            [
                cx - rect_width // 2 + x_off,
                cy - rect_height // 2 + y_off,
                cx + rect_width // 2 + x_off,
                cy + rect_height // 2 + y_off,
            ],
            radius=int(size * 0.08),
            fill=shade,
        )


def draw_sliders(draw, cx, cy, size, color):
    line_width = int(size * 0.06)
    line_length = size * 0.7

    gray = (180, 180, 180)

    for i, y_offset in enumerate([-size * 0.25, 0, size * 0.25]):
        y = int(cy + y_offset)
        draw.rounded_rectangle(
            [
                cx - line_length // 2,
                y - line_width // 2,
                cx + line_length // 2,
                y + line_width // 2,
            ],
            radius=line_width // 2,
            fill=gray,
        )

        knob_x = int(cx - line_length // 4 + i * size * 0.2)
        knob_radius = int(size * 0.1)
        draw.ellipse(
            [
                knob_x - knob_radius,
                y - knob_radius,
                knob_x + knob_radius,
                y + knob_radius,
            ],
            fill=color,
        )


def draw_sound_waves(draw, cx, cy, size, color):
    bar_width = int(size * 0.08)
    gap = int(size * 0.04)
    bars = 5
    total_width = bars * bar_width + (bars - 1) * gap
    start_x = int(cx - total_width // 2)

    heights = [0.3, 0.6, 1.0, 0.7, 0.4]
    for i, h in enumerate(heights):
        x = start_x + i * (bar_width + gap)
        bar_height = int(size * 0.7 * h)
        draw.rounded_rectangle(
            [x, int(cy - bar_height // 2), x + bar_width, int(cy + bar_height // 2)],
            radius=bar_width // 2,
            fill=color,
        )


def draw_refresh_arrows(draw, cx, cy, size, color):
    radius = size * 0.35
    arrow_size = int(size * 0.12)
    line_width = int(size * 0.08)

    bbox = [
        cx - radius - line_width,
        cy - radius - line_width,
        cx + radius + line_width,
        cy + radius + line_width,
    ]

    draw.arc(bbox, start=45, end=270, fill=color, width=line_width)
    draw.arc(bbox, start=225, end=450, fill=color, width=line_width)

    draw.polygon(
        [
            (cx + radius, cy - arrow_size),
            (cx + radius + arrow_size * 1.5, cy),
            (cx + radius, cy + arrow_size),
        ],
        fill=color,
    )

    draw.polygon(
        [
            (cx - radius, cy + arrow_size),
            (cx - radius - arrow_size * 1.5, cy),
            (cx - radius, cy - arrow_size),
        ],
        fill=color,
    )


STYLES = [
    ("gear", draw_gear, (0, 150, 200)),
    ("bell", draw_notification_bell, (200, 80, 80)),
    ("layers", draw_layers, (80, 180, 80)),
    ("sliders", draw_sliders, (180, 120, 200)),
    ("waves", draw_sound_waves, (220, 160, 40)),
    ("refresh", draw_refresh_arrows, (60, 100, 180)),
]


def create_icon(size: int, style_index: int = 0) -> Image.Image:
    img = Image.new("RGBA", (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)

    name, draw_func, color = STYLES[style_index % len(STYLES)]

    margin = size // 8
    icon_size = size - margin * 2
    cx, cy = size // 2, size // 2

    draw_func(draw, cx, cy, icon_size // 2, color)

    return img


def main():
    import random

    style_index = random.randint(0, len(STYLES) - 1)
    selected_style = STYLES[style_index][0]
    print(f"Selected style: {selected_style}")

    for size in SIZES:
        size_dir = f"{OUTPUT_DIR}/{size}x{size}"
        os.makedirs(size_dir, exist_ok=True)

        img = create_icon(size, style_index=style_index)
        output_path = f"{size_dir}/{ICON_NAME}.png"
        img.save(output_path, "PNG")
        print(f"Created: {output_path}")


if __name__ == "__main__":
    main()
