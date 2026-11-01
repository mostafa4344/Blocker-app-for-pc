"""Renders the tutorial: frames -> ffmpeg (H.264 1080p30) + voice/clicks mix + subtitles.

  python render.py still <seconds>...   preview frames as PNG
  python render.py video [from to]      render (optionally a time range)
"""
import json, math, os, subprocess, sys, wave
import numpy as np
from PIL import Image, ImageDraw, ImageFilter, ImageFont

sys.stdout.reconfigure(encoding="utf-8")
D = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, D)
from script import S, CHAPTERS  # noqa: E402

W, H, FPS = 1920, 1080, 30
BRAND = (79, 70, 229)
LEAD, TAIL, XF = 0.45, 0.6, 0.35
SHOTS = f"{D}/shots"
FONT = "C:/Windows/Fonts/"
_f = {}


def font(name, size):
    k = (name, int(size))
    if k not in _f:
        _f[k] = ImageFont.truetype(FONT + name, int(size))
    return _f[k]


def ease(x):
    x = min(1.0, max(0.0, x))
    return x * x * (3 - 2 * x)


def ease_out(x):
    x = min(1.0, max(0.0, x))
    return 1 - (1 - x) ** 3


# ---------------------------------------------------------------- timeline
META = [json.load(open(f"{D}/voice/s{i:02d}.json", encoding="utf-8")) for i in range(len(S))]
STARTS, t = [], 0.0
for i, sc in enumerate(S):
    STARTS.append(t)
    sc["dur"] = LEAD + META[i]["dur"] + TAIL + (0.8 if sc["frame"] in ("title", "end") else 0)
    t += sc["dur"]
TOTAL = t
CH_INDEX = {k: i for i, (k, _) in enumerate(CHAPTERS)}
CH_TITLE = dict(CHAPTERS)
FIRST_OF_CH = {}
for i, sc in enumerate(S):
    FIRST_OF_CH.setdefault(sc["ch"], i)

# ---------------------------------------------------------------- assets
_img = {}


def img(name):
    if name not in _img:
        _img[name] = Image.open(f"{SHOTS}/{name}.png").convert("RGBA" if name in ("ss1", "ss2", "uac") else "RGB")
    return _img[name]


def marks(name):
    p = f"{SHOTS}/{name}.json"
    return json.load(open(p)) if os.path.exists(p) else {}


LOGO = Image.open(f"{D}/logo.png").convert("RGBA")


def gradient(w, h, c0, c1, diag=True):
    a = np.linspace(0, 1, w)[None, :] * 0.6 + np.linspace(0, 1, h)[:, None] * 0.4 if diag else np.linspace(0, 1, h)[:, None] * np.ones((1, w))
    c0, c1 = np.array(c0, float), np.array(c1, float)
    arr = c0[None, None, :] * (1 - a[..., None]) + c1[None, None, :] * a[..., None]
    return Image.fromarray(arr.astype(np.uint8), "RGB")


def soft_bg():
    bg = gradient(W, H, (236, 238, 252), (247, 245, 255))
    blob = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(blob)
    d.ellipse((-300, -350, 700, 500), fill=90)
    d.ellipse((1300, 600, 2300, 1500), fill=70)
    blob = blob.filter(ImageFilter.GaussianBlur(160))
    tint = Image.new("RGB", (W, H), (167, 160, 250))
    return Image.composite(tint, bg, blob)


def desktop_bg():
    bg = gradient(W, H, (8, 42, 105), (19, 101, 196))
    glow = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(glow)
    d.polygon([(1150, 0), (1500, 0), (900, H), (500, H)], fill=60)
    d.polygon([(1550, 0), (1700, 0), (1250, H), (1050, H)], fill=35)
    glow = glow.filter(ImageFilter.GaussianBlur(60))
    bg = Image.composite(Image.new("RGB", (W, H), (120, 190, 255)), bg, glow)
    d = ImageDraw.Draw(bg)
    d.rectangle((0, H - 48, W, H), fill=(22, 28, 40))
    d.rectangle((0, H - 48, 48, H), fill=(30, 38, 54))
    for k, x in enumerate((14, 22)):
        pass
    # Windows logo (four panes) + a few pinned icons
    for dx, dy in ((14, 1043), (25, 1043), (14, 1054), (25, 1054)):
        d.rectangle((dx, dy, dx + 9, dy + 9), fill=(255, 255, 255))
    d.rounded_rectangle((60, H - 40, 380, H - 8), 3, fill=(52, 60, 76))
    d.text((74, H - 33), "Type here to search", font=font("segoeui.ttf", 15), fill=(200, 205, 215))
    t = font("segoeui.ttf", 14)
    d.text((W - 92, H - 42), "10:30 AM", font=t, fill="white")
    d.text((W - 96, H - 24), "11/1/2026", font=t, fill="white")
    return bg


def title_bg(t):
    bg = gradient(W, H, (67, 56, 202), (124, 58, 237))
    blob = Image.new("L", (W, H), 0)
    d = ImageDraw.Draw(blob)
    x = 300 + 120 * math.sin(t * 0.4)
    d.ellipse((x - 500, -400, x + 500, 600), fill=70)
    d.ellipse((1500 - x * 0.3, 600, 2400 - x * 0.3, 1500), fill=60)
    blob = blob.filter(ImageFilter.GaussianBlur(140))
    return Image.composite(Image.new("RGB", (W, H), (190, 180, 255)), bg, blob)


SOFT_BG = soft_bg()
DESK_BG = desktop_bg()
TITLE_BG = [title_bg(k * 0.5) for k in range(40)]

# ---------------------------------------------------------------- window chrome (pre-rendered at 2x)
S2 = 2


def app_bar(w):
    im = Image.new("RGBA", (w * S2, 36 * S2), (255, 255, 255, 255))
    d = ImageDraw.Draw(im)
    lg = LOGO.resize((20 * S2, 20 * S2), Image.LANCZOS)
    im.paste(lg, (12 * S2, 8 * S2), lg)
    d.text((40 * S2, 8 * S2), "HTL Content Guard PC", font=font("segoeui.ttf", 13 * S2), fill=(40, 40, 50))
    for k, ch in enumerate(("—", "☐", "✕")):
        d.text(((w - 130 + k * 46) * S2, 6 * S2), ch, font=font("seguisym.ttf" if ch != "—" else "segoeui.ttf", 13 * S2) if os.path.exists(FONT + "seguisym.ttf") else font("segoeui.ttf", 13 * S2), fill=(60, 60, 70))
    d.line((0, 36 * S2 - 1, w * S2, 36 * S2 - 1), fill=(225, 227, 235), width=2)
    return im


def browser_bar(w, tab, url):
    im = Image.new("RGBA", (w * S2, 72 * S2), (223, 225, 230, 255))
    d = ImageDraw.Draw(im)
    d.rounded_rectangle((10 * S2, 6 * S2, 250 * S2, 40 * S2), 9 * S2, fill=(255, 255, 255))
    d.rectangle((10 * S2, 28 * S2, 250 * S2, 36 * S2), fill=(255, 255, 255))
    lg = LOGO.resize((16 * S2, 16 * S2), Image.LANCZOS)
    im.paste(lg, (22 * S2, 14 * S2), lg)
    d.text((44 * S2, 13 * S2), tab, font=font("segoeui.ttf", 12 * S2), fill=(40, 40, 50))
    d.text((228 * S2, 12 * S2), "✕", font=font("segoeui.ttf", 11 * S2), fill=(90, 90, 100))
    d.rectangle((0, 36 * S2, w * S2, 72 * S2), fill=(255, 255, 255))
    for k, ch in enumerate(("←", "→", "⟳")):
        d.text(((16 + k * 30) * S2, 42 * S2), ch, font=font("segoeui.ttf", 15 * S2), fill=(95, 99, 104))
    d.rounded_rectangle((110 * S2, 41 * S2, (w - 60) * S2, 67 * S2), 13 * S2, fill=(241, 243, 244))
    d.text((128 * S2, 45 * S2), "🔒", font=font("seguiemj.ttf", 11 * S2), embedded_color=True)
    d.text((152 * S2, 45 * S2), url, font=font("segoeui.ttf", 13 * S2), fill=(32, 33, 36))
    return im


# Layout per frame type: content rect V (virtual px), frame top, chrome image.
def layout(sc):
    f = sc["frame"]
    if f == "app":
        return dict(V=(160, 108, 1600, 900), top=72, chrome=APP_BAR, bg=SOFT_BG)
    if f in ("web", "store", "block"):
        return dict(V=(160, 140, 1600, 900), top=68, chrome=BROWSER_BARS[f], bg=SOFT_BG)
    if f == "installer":
        im = img(sc["shot"])
        k = 2.0
        w, h = im.width * k, im.height * k
        return dict(V=((W - w) / 2, (H - 48 - h) / 2, w, h), top=(H - 48 - h) / 2, chrome=None, bg=DESK_BG)
    if f == "dialog":
        im = img(sc["shot"])
        if sc["shot"] == "uac":
            w = 720
        else:
            w = 880
        h = w * im.height / im.width
        return dict(V=((W - w) / 2, (H - 48 - h) / 2, w, h), top=(H - 48 - h) / 2, chrome=None, bg=DESK_BG)
    return dict(V=(0, 0, W, H), top=0, chrome=None, bg=None)


APP_BAR = app_bar(1600)
BROWSER_BARS = {
    "web": browser_bar(1600, "HTL Content Guard PC", "HTL Content Guard PC — free download"),
    "store": browser_bar(1600, "Chrome Web Store", "chromewebstore.google.com/detail/htl-content-guard-pc/amlkhggimheblaboilcbjdjomlceckdp"),
    "block": browser_bar(1600, "HTL Content Guard", "chrome-extension://amlkhggimheblaboilcbjdjomlceckdp/blocked.html"),
}


def shadow_img(w, h, r=14, blur=28, alpha=70):
    pad = blur * 3
    m = Image.new("L", (int(w + pad * 2), int(h + pad * 2)), 0)
    ImageDraw.Draw(m).rounded_rectangle((pad, pad + 12, pad + w, pad + h + 12), r, fill=alpha)
    rgba = Image.new("RGBA", m.size, (20, 18, 50, 0))
    rgba.putalpha(m.filter(ImageFilter.GaussianBlur(blur)))
    return rgba, pad


_shadow = {}

# ---------------------------------------------------------------- camera


def T(p, cam):
    cx, cy, z = cam
    return ((p[0] - cx) * z + W / 2, (p[1] - cy) * z + H / 2)


def draw_scaled(base, src, vrect, cam, mask=None, resample=Image.BICUBIC):
    """Paste src (any size) so it covers virtual rect vrect, under camera cam."""
    x, y, w, h = vrect
    (sx0, sy0), (sx1, sy1) = T((x, y), cam), T((x + w, y + h), cam)
    dx0, dy0, dx1, dy1 = max(0, sx0), max(0, sy0), min(W, sx1), min(H, sy1)
    if dx1 - dx0 < 1 or dy1 - dy0 < 1:
        return
    kx, ky = src.width / (sx1 - sx0), src.height / (sy1 - sy0)
    box = ((dx0 - sx0) * kx, (dy0 - sy0) * ky, (dx1 - sx0) * kx, (dy1 - sy0) * ky)
    ox, oy = int(round(dx0)), int(round(dy0))
    size = (int(round(dx1)) - ox, int(round(dy1)) - oy)
    if size[0] < 1 or size[1] < 1:
        return
    part = src.resize(size, resample, box=box, reducing_gap=2.0)
    if mask is not None:
        base.paste(mask[0], (ox, oy), part if mask[1] == "self" else part)
    elif part.mode == "RGBA":
        base.paste(part, (ox, oy), part)
    else:
        base.paste(part, (ox, oy))


def frame_extent(L):
    x, y, w, h = L["V"]
    return (x, L["top"], x + w, y + h)


def cam_for(sc, L, box=None, zoom=1.0):
    if box is None or zoom <= 1.0:
        return (W / 2, H / 2, 1.0)
    x, y, w, h = L["V"]
    cx, cy = x + (box[0] + box[2] / 2) * w, y + (box[1] + box[3] / 2) * h
    return clamp_cam((cx, cy, zoom), L)


def clamp_cam(cam, L):
    cx, cy, z = cam
    x0, y0, x1, y1 = frame_extent(L)
    hw, hh = W / 2 / z, H / 2 / z
    pad = 24 / z
    x0, y0, x1, y1 = x0 - pad, y0 - pad, x1 + pad, y1 + pad
    cx = (x0 + x1) / 2 if x1 - x0 <= 2 * hw else min(max(cx, x0 + hw), x1 - hw)
    cy = (y0 + y1) / 2 if y1 - y0 <= 2 * hh else min(max(cy, y0 + hh), y1 - hh)
    return (cx, cy, z)


def lerp_cam(a, b, k):
    za, zb = math.log(a[2]), math.log(b[2])
    return (a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k, math.exp(za + (zb - za) * k))


def vbox(L, b):
    x, y, w, h = L["V"]
    return (x + b[0] * w, y + b[1] * h, b[2] * w, b[3] * h)


# Per-scene plan: camera keyframes, rings, clicks.
PLANS = []


def scene_marks(sc):
    if sc["frame"] == "installer" and sc.get("btn"):
        return {"btn": sc["btn"]}
    return marks(sc["shot"]) if sc.get("shot") else {}


prev_cam, prev_shot, prev_cursor = None, None, None
for i, sc in enumerate(S):
    L = layout(sc)
    mk = scene_marks(sc)
    full = (W / 2, H / 2, 1.0)
    focus = sc.get("focusbox") or (mk.get(sc.get("focus")) if sc.get("focus") else None)
    target = cam_for(sc, L, focus, sc.get("zoom", 1.0))
    same = prev_shot is not None and prev_shot == sc.get("shot")
    keys = [(0.0, prev_cam if same else full)]
    keys.append((0.25 if not same else 0.0, keys[-1][1]))
    keys.append((1.45 if not same else 1.0, target))
    rings, clicks = [], []
    cam_now = target
    for m in META[i]["marks"]:
        if m["key"] not in mk:
            continue
        tm = LEAD + m["t"]
        b = vbox(L, mk[m["key"]])
        # Pan so the mark is comfortably in view.
        cx, cy, z = cam_now
        hw, hh = W / 2 / z * 0.82, H / 2 / z * 0.82
        if z > 1.0 and (b[0] < cx - hw or b[0] + b[2] > cx + hw or b[1] < cy - hh or b[1] + b[3] > cy + hh):
            new = clamp_cam((b[0] + b[2] / 2, b[1] + b[3] / 2, z), L)
            t0 = max(keys[-1][0] + 0.05, tm - 0.9)
            keys.append((t0, cam_now))
            keys.append((t0 + 0.8, new))
            cam_now = new
        rings.append((tm, b))
        if m["click"]:
            clicks.append((tm, (b[0] + b[2] * 0.5, b[1] + b[3] * 0.55)))
    PLANS.append(dict(L=L, keys=sorted(keys, key=lambda k: k[0]), rings=rings, clicks=clicks,
                      cursor0=prev_cursor if same and prev_cursor else None))
    prev_cam, prev_shot = cam_now, sc.get("shot")
    prev_cursor = clicks[-1][1] if clicks else (prev_cursor if same else None)


def cam_at(plan, t):
    ks = plan["keys"]
    if t <= ks[0][0]:
        return ks[0][1]
    for (t0, c0), (t1, c1) in zip(ks, ks[1:]):
        if t0 <= t <= t1:
            return lerp_cam(c0, c1, ease((t - t0) / (t1 - t0)) if t1 > t0 else 1)
    return ks[-1][1]


# ---------------------------------------------------------------- overlays
CURSOR = None


def make_cursor():
    s = 3
    pts = [(0, 0), (0, 21), (5, 16), (9, 25), (12, 24), (8, 15), (15, 15)]
    im = Image.new("RGBA", (22 * s, 30 * s), (0, 0, 0, 0))
    d = ImageDraw.Draw(im)
    sh = Image.new("RGBA", im.size, (0, 0, 0, 0))
    ImageDraw.Draw(sh).polygon([(x * s + 4, y * s + 5) for x, y in pts], fill=(0, 0, 0, 90))
    sh = sh.filter(ImageFilter.GaussianBlur(4))
    im.alpha_composite(sh)
    d.polygon([(x * s + 1, y * s + 1) for x, y in pts], fill=(255, 255, 255, 255), outline=(0, 0, 0, 255), width=s)
    return im.resize((22, 30), Image.LANCZOS)


CURSOR = make_cursor()


def draw_ring(base, rect, alpha, grow):
    x, y, w, h = rect
    pad = 8 + 14 * grow
    x0, y0, x1, y1 = x - pad, y - pad, x + w + pad, y + h + pad
    if x1 < 0 or y1 < 0 or x0 > W or y0 > H or alpha <= 0.01:
        return
    m = 30
    lw, lh = int(x1 - x0 + 2 * m), int(y1 - y0 + 2 * m)
    if lw > 2 * W or lh > 2 * H:
        return
    layer = Image.new("RGBA", (lw, lh), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    r = min(16, (y1 - y0) / 2)
    d.rounded_rectangle((m, m, lw - m, lh - m), r, outline=BRAND + (int(150 * alpha),), width=12)
    layer = layer.filter(ImageFilter.GaussianBlur(7))
    d = ImageDraw.Draw(layer)
    d.rounded_rectangle((m, m, lw - m, lh - m), r, outline=BRAND + (int(255 * alpha),), width=4)
    base.paste(layer, (int(x0 - m), int(y0 - m)), layer)


def draw_ripple(base, p, k):
    if not 0 <= k <= 1:
        return
    rad = 8 + 34 * ease_out(k)
    a = int(160 * (1 - k))
    layer = Image.new("RGBA", (120, 120), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    d.ellipse((60 - rad, 60 - rad, 60 + rad, 60 + rad), fill=BRAND + (a // 3,), outline=BRAND + (a,), width=3)
    base.paste(layer, (int(p[0] - 60), int(p[1] - 60)), layer)


def pill(base, text, alpha):
    if alpha <= 0.01:
        return
    f = font("seguisb.ttf", 20)
    tw = f.getlength(text)
    w, h = int(tw + 44), 40
    layer = Image.new("RGBA", (w + 40, h + 40), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    d.rounded_rectangle((20, 24, 20 + w, 24 + h), 20, fill=(0, 0, 0, int(40 * alpha)))
    layer = layer.filter(ImageFilter.GaussianBlur(8))
    d = ImageDraw.Draw(layer)
    d.rounded_rectangle((20, 20, 20 + w, 20 + h), 20, fill=(255, 255, 255, int(245 * alpha)))
    d.ellipse((36, 34, 48, 46), fill=BRAND + (int(255 * alpha),))
    d.text((58, 26), text, font=f, fill=(30, 30, 46, int(255 * alpha)))
    base.paste(layer, (8, 2), layer)


def lower_third(base, step, title, k):
    """k: 0..1 over its life (in, hold, out)."""
    a = ease_out(k / 0.18) if k < 0.18 else (1 if k < 0.85 else 1 - ease((k - 0.85) / 0.15))
    if a <= 0.01:
        return
    dy = (1 - a) * 30
    f1, f2 = font("seguisb.ttf", 22), font("seguisb.ttf", 46)
    w = int(max(f1.getlength(step), f2.getlength(title)) + 90)
    h = 128
    layer = Image.new("RGBA", (w + 60, h + 60), (0, 0, 0, 0))
    d = ImageDraw.Draw(layer)
    d.rounded_rectangle((30, 38, 30 + w, 38 + h), 22, fill=(20, 16, 60, int(70 * a)))
    layer = layer.filter(ImageFilter.GaussianBlur(14))
    d = ImageDraw.Draw(layer)
    d.rounded_rectangle((30, 30, 30 + w, 30 + h), 22, fill=(255, 255, 255, int(250 * a)))
    d.rounded_rectangle((30, 30, 40, 30 + h), 5, fill=BRAND + (int(255 * a),))
    d.text((66, 46), step, font=f1, fill=BRAND + (int(255 * a),))
    d.text((64, 76), title, font=f2, fill=(24, 24, 40, int(255 * a)))
    base.paste(layer, (12, int(46 - dy)), layer)


def progress(base, t):
    d = ImageDraw.Draw(base)
    d.rectangle((0, H - 5, W, H), fill=(214, 216, 235))
    d.rectangle((0, H - 5, int(W * t / TOTAL), H), fill=BRAND)


# ---------------------------------------------------------------- scene renderers


def render_title(i, lt, end=False):
    k = int((STARTS[i] + lt) * 2) % len(TITLE_BG)
    base = TITLE_BG[k].copy()
    a = ease_out(lt / 0.9)
    size = int(220 + 30 * (1 - a))
    lg = LOGO.resize((size, size), Image.LANCZOS)
    if a < 1:
        al = lg.split()[3].point(lambda v: int(v * a))
        lg.putalpha(al)
    cy = 330 if not end else 300
    base.paste(lg, (W // 2 - size // 2, cy - size // 2), lg)
    d = ImageDraw.Draw(base, "RGBA")

    def ctext(y, s, f, col, delay):
        b = ease_out((lt - delay) / 0.7)
        if b <= 0:
            return
        w = f.getlength(s)
        d.text((W / 2 - w / 2, y + (1 - b) * 24), s, font=f, fill=col + (int(255 * b),))

    if not end:
        ctext(480, "HTL Content Guard PC", font("seguisb.ttf", 96), (255, 255, 255), 0.3)
        ctext(612, "Complete setup guide", font("segoeuil.ttf", 50), (235, 233, 255), 0.6)
        ctext(720, "Install  ·  Set up  ·  Block  ·  Stay focused", font("segoeui.ttf", 30), (210, 205, 255), 0.9)
    else:
        ctext(450, "Thank you for watching", font("seguisb.ttf", 84), (255, 255, 255), 0.2)
        ctext(570, "Download free  ·  4-day free trial", font("segoeuil.ttf", 46), (235, 233, 255), 0.5)
        ctext(680, "WhatsApp +880 1568-689505   ·   contentguard.support@gmail.com", font("segoeui.ttf", 30), (215, 210, 255), 0.8)
        ctext(740, "HTL Content Guard PC  ·  Halal Tech Life", font("segoeui.ttf", 26), (200, 195, 250), 1.0)
    return base


def render_scene(i, lt):
    sc = S[i]
    if sc["frame"] in ("title", "end"):
        return render_title(i, lt, sc["frame"] == "end")
    plan = PLANS[i]
    L = plan["L"]
    cam = cam_at(plan, lt)
    base = L["bg"].copy()
    if sc["frame"] == "dialog" and sc.get("dim"):
        base = Image.blend(base, Image.new("RGB", (W, H), (0, 0, 0)), 0.45)
    x, y, w, h = L["V"]
    top = L["top"]
    # shadow
    key = (w, h + (y - top))
    if key not in _shadow:
        _shadow[key] = shadow_img(*key)
    sm, pad = _shadow[key]
    if cam[2] < 1.6:
        draw_scaled(base, sm, (x - pad, top - pad, w + 2 * pad, h + (y - top) + 2 * pad), cam)
    if L["chrome"] is not None:
        draw_scaled(base, L["chrome"], (x, top, w, y - top), cam)
    draw_scaled(base, img(sc["shot"]), (x, y, w, h), cam)
    if sc["frame"] in ("app", "web", "store", "block"):
        # thin border
        (bx0, by0), (bx1, by1) = T((x, top), cam), T((x + w, y + h), cam)
        ImageDraw.Draw(base).rectangle((bx0 - 1, by0 - 1, bx1, by1), outline=(205, 208, 225), width=1)
    # rings: each stays until the next one appears
    for j, (tm, b) in enumerate(plan["rings"]):
        if lt < tm:
            continue
        nxt = plan["rings"][j + 1][0] if j + 1 < len(plan["rings"]) else None
        a = ease_out((lt - tm) / 0.35)
        if nxt is not None and lt > nxt:
            a *= 1 - ease((lt - nxt) / 0.3)
        (sx, sy) = T((b[0], b[1]), cam)
        draw_ring(base, (sx, sy, b[2] * cam[2], b[3] * cam[2]), a, 1 - ease_out((lt - tm) / 0.35))
    # cursor + clicks
    if plan["clicks"]:
        x0, y0, w0, h0 = L["V"]
        # Start beside the window (on prompts, never resting on a wrong button).
        outside = sc["frame"] in ("dialog", "installer")
        start = plan["cursor0"] or ((x0 + w0 + 60, y0 + h0 * 0.6) if outside else (x0 + w0 * 0.78, y0 + h0 * 0.86))
        pos, prev, pressed = start, start, 0
        for tm, p in plan["clicks"]:
            t_arrive = tm - 0.08
            t_leave = max(t_arrive - 0.75, 0.2)
            if lt < t_leave:
                pos = prev
                break
            if lt < t_arrive:
                k = ease((lt - t_leave) / (t_arrive - t_leave))
                pos = (prev[0] + (p[0] - prev[0]) * k, prev[1] + (p[1] - prev[1]) * k)
                break
            pos, prev = p, p
            if lt - tm < 0.6:
                draw_ripple(base, T(p, cam), (lt - tm) / 0.5)
            if 0 <= lt - tm < 0.14:
                pressed = 1
        sp = T(pos, cam)
        cur = CURSOR if not pressed else CURSOR.resize((19, 26), Image.LANCZOS)
        fade = ease_out(lt / 0.4) if plan["cursor0"] is None else 1
        if fade < 1:
            cur = cur.copy()
            cur.putalpha(cur.split()[3].point(lambda v: int(v * fade)))
        base.paste(cur, (int(sp[0]), int(sp[1])), cur)
    return base


def frame_at(t):
    i = max(k for k, s in enumerate(STARTS) if s <= t) if t >= 0 else 0
    lt = t - STARTS[i]
    im = render_scene(i, lt)
    end = STARTS[i] + S[i]["dur"]
    if i + 1 < len(S) and t > end - XF and S[i + 1].get("shot") != S[i].get("shot"):
        nxt = render_scene(i + 1, 0.0)
        im = Image.blend(im, nxt, ease((t - (end - XF)) / XF))
    sc = S[i]
    if sc["frame"] not in ("title", "end"):
        first = FIRST_OF_CH[sc["ch"]]
        ch_t = t - STARTS[first]
        steps = [k for k, _ in CHAPTERS if k not in ("intro", "help")]
        title = CH_TITLE[sc["ch"]]
        label = title.split(" · ", 1)[-1]
        if ch_t < 3.2 and sc["ch"] != "intro":
            n = steps.index(sc["ch"]) + 1 if sc["ch"] in steps else None
            lower_third(im, f"STEP {n} OF {len(steps)}" if n else "", label, ch_t / 3.2)
        pill(im, title if sc["ch"] != "intro" else "HTL Content Guard PC", ease((ch_t - 2.6) / 0.5) if sc["ch"] != "intro" else 1)
        progress(im, t)
    return im


# ---------------------------------------------------------------- audio + subtitles


def click_sound(sr=48000):
    n = int(sr * 0.035)
    tt = np.arange(n) / sr
    s = np.sin(2 * np.pi * 2300 * tt) * np.exp(-tt * 180) * 0.35 + np.random.default_rng(1).normal(0, 1, n) * np.exp(-tt * 400) * 0.08
    return s.astype(np.float32)


def build_audio(path):
    sr = 48000
    buf = np.zeros(int((TOTAL + 1) * sr), np.float32)
    clk = click_sound(sr)
    for i in range(len(S)):
        raw = subprocess.run(["ffmpeg", "-v", "error", "-i", f"{D}/voice/s{i:02d}.mp3", "-f", "f32le", "-ac", "1", "-ar", str(sr), "-"], capture_output=True).stdout
        v = np.frombuffer(raw, np.float32)
        o = int((STARTS[i] + LEAD) * sr)
        buf[o:o + len(v)] += v
        for tm, _ in PLANS[i]["clicks"]:
            o = int((STARTS[i] + tm) * sr)
            buf[o:o + len(clk)] += clk
    buf = np.clip(buf, -1, 1)
    with wave.open(path, "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(sr)
        w.writeframes((buf * 32767).astype(np.int16).tobytes())


def ts(x):
    ms = int(round(x * 1000))
    return f"{ms // 3600000:02d}:{ms // 60000 % 60:02d}:{ms // 1000 % 60:02d},{ms % 1000:03d}"


def build_srt(lang, path):
    out, n = [], 1
    for i in range(len(S)):
        for s in META[i]["sents"]:
            a = STARTS[i] + LEAD + s["t0"]
            b = STARTS[i] + LEAD + s["t1"] + 0.25
            out.append(f"{n}\n{ts(a)} --> {ts(b)}\n{s[lang]}\n")
            n += 1
    open(path, "w", encoding="utf-8").write("\n".join(out))


def chapters_txt(path):
    lines = []
    for k, title in CHAPTERS:
        t0 = STARTS[FIRST_OF_CH[k]]
        lines.append(f"{int(t0 // 60)}:{int(t0 % 60):02d} {title.split(' · ', 1)[-1] if k not in ('intro',) else 'Welcome'}")
    open(path, "w", encoding="utf-8").write("\n".join(lines) + "\n")


# ---------------------------------------------------------------- main
if __name__ == "__main__":
    cmd = sys.argv[1]
    print(f"total {TOTAL:.1f}s, {len(S)} scenes")
    if cmd == "still":
        for a in sys.argv[2:]:
            frame_at(float(a)).save(f"{D}/still_{a}.png")
    elif cmd == "scene":  # middle of each listed scene
        for a in sys.argv[2:]:
            i = int(a)
            frame_at(STARTS[i] + S[i]["dur"] * 0.7).save(f"{D}/scene_{i:02d}.png")
    elif cmd == "video":
        t0 = float(sys.argv[2]) if len(sys.argv) > 2 else 0
        t1 = float(sys.argv[3]) if len(sys.argv) > 3 else TOTAL
        out = f"{D}/out"
        os.makedirs(out, exist_ok=True)
        silent = f"{out}/video_{int(t0)}_{int(t1)}.mp4"
        p = subprocess.Popen(["ffmpeg", "-y", "-v", "error", "-f", "rawvideo", "-pix_fmt", "rgb24", "-s", f"{W}x{H}", "-r", str(FPS), "-i", "-",
                              "-c:v", "libx264", "-preset", "medium", "-crf", "17", "-pix_fmt", "yuv420p", "-movflags", "+faststart", silent], stdin=subprocess.PIPE)
        n0, n1 = int(t0 * FPS), int(t1 * FPS)
        import time
        st = time.time()
        for n in range(n0, n1):
            p.stdin.write(frame_at(n / FPS).convert("RGB").tobytes())
            if n % 300 == 0:
                el = time.time() - st
                print(f"frame {n}/{n1}  {el:.0f}s", flush=True)
        p.stdin.close()
        p.wait()
        if t0 == 0 and t1 == TOTAL:
            build_audio(f"{out}/audio.wav")
            build_srt("en", f"{out}/HTL-Content-Guard-PC-Tutorial.en.srt")
            build_srt("bn", f"{out}/HTL-Content-Guard-PC-Tutorial.bn.srt")
            chapters_txt(f"{out}/chapters.txt")
        print("done", silent)
