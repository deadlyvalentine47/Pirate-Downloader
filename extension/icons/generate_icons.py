from PIL import Image, ImageDraw, ImageFont
import os

def create_icon(size):
    # Create image with transparent background
    img = Image.new('RGBA', (size, size), (0, 0, 0, 0))
    draw = ImageDraw.Draw(img)
    
    # Draw circle
    draw.ellipse((0, 0, size, size), fill='#007acc')
    
    # Draw text "PD"
    # Basic attempt at centering text - Pillow font handling is tricky without a .ttf file
    # We'll just draw a simple shape for now to avoid font issues
    margin = size // 4
    draw.rectangle((margin, margin, size - margin, size - margin), fill='white')

    img.save(f'icon{size}.png')
    print(f'Created icon{size}.png')

sizes = [16, 48, 128]

if __name__ == "__main__":
    for size in sizes:
        create_icon(size)
