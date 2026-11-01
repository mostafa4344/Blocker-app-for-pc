import os, sys
from PIL import Image, ImageDraw, ImageFilter, ImageFont
import numpy as np

D = os.path.dirname(os.path.abspath(__file__))
W, H = 1280, 720
F = "C:/Windows/Fonts/"


def font(n, s):
    return ImageFont.truetype(F + n, s)


a = np.linspace(0, 1, W)[None, :] * 0.6 + np.linspace(0, 1, H)[:, None] * 0.4
c0, c1 = np.array((55, 48, 190.0)), np.array((126, 58, 237.0))
bg = Image.fromarray((c0 * (1 - a[..., None]) + c1 * a[..., None]).astype(np.uint8))
glow = Image.new("L", (W, H), 0)
ImageDraw.Draw(glow).ellipse((700, -200, 1500, 600), fill=90)
glow = glow.filter(ImageFilter.GaussianBlur(120))
bg = Image.composite(Image.new("RGB", (W, H), (200, 190, 255)), bg, glow)

# app screenshot card on the right, slightly rotated
shot = Image.open(f"{D}/shots/home_premium.png").convert("RGB").resize((760, 428), Image.LANCZOS)
card = Image.new("RGBA", (shot.width + 16, shot.height + 16), (255, 255, 255, 255))
card.paste(shot, (8, 8))
card = card.rotate(-6, resample=Image.BICUBIC, expand=True)
sh = Image.new("RGBA", card.size, (0, 0, 0, 0))
sh.putalpha(card.split()[3].point(lambda v: int(v * 0.45)))
sh = sh.filter(ImageFilter.GaussianBlur(18))
bg.paste((10, 8, 40), (590, 250), sh)
bg.paste(card, (575, 225), card)

d = ImageDraw.Draw(bg)
logo = Image.open(f"{D}/logo.png").convert("RGBA").resize((96, 96), Image.LANCZOS)
bg.paste(logo, (60, 56), logo)
d.text((172, 70), "HTL Content Guard PC", font=font("seguisb.ttf", 44), fill="white")
d.text((60, 190), "INSTALL &", font=font("segoeuib.ttf", 96), fill="white")
d.text((60, 292), "SET UP", font=font("segoeuib.ttf", 96), fill=(255, 226, 120))
d.text((64, 404), "Full guide · 7 minutes", font=font("seguisb.ttf", 38), fill=(230, 226, 255))
d.rounded_rectangle((60, 486, 470, 560), 37, fill=(255, 255, 255))
d.text((92, 498), "FREE 4-DAY TRIAL", font=font("segoeuib.ttf", 38), fill=(79, 70, 229))
bg.save(f"{D}/out/thumbnail.png")
bg.convert("RGB").save(f"{D}/out/thumbnail.jpg", quality=92)
print("ok")
