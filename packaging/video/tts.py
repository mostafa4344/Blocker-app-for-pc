"""Voice-over for every scene + exact timings (markers and sentences)."""
import asyncio, json, os, re, subprocess, sys
import edge_tts

sys.stdout.reconfigure(encoding="utf-8")
sys.path.insert(0, os.path.dirname(__file__))
from script import S, SAY  # noqa: E402
from script_bn import BN  # noqa: E402

LANG = os.environ.get("CG_LANG", "en")

VOICE = "bn-BD-PradeepNeural" if LANG == "bn" else "en-US-AndrewMultilingualNeural"
RATE = "+0%" if LANG == "bn" else "-3%"
OUT = os.path.join(os.path.dirname(__file__), "voice_bn" if LANG == "bn" else "voice")
os.makedirs(OUT, exist_ok=True)
MARK = re.compile(r"\[\[(\w+)(!?)\]\]")


def say(s):
    for a, b in SAY.items():
        s = re.sub(rf"(?<![\w-]){re.escape(a)}(?![\w-])", b, s)
    return s


def plan(text):
    """Spoken text, marker char positions, sentence spans (start, end, display)."""
    sentences = re.split(r"(?<=[.!?।])\s+", text.strip())
    spoken, marks, spans = "", [], []
    for sent in sentences:
        if spoken:
            spoken += " "
        start = len(spoken)
        pos = 0
        for m in MARK.finditer(sent):
            spoken += say(sent[pos:m.start()])
            marks.append((m.group(1), m.group(2) == "!", len(spoken)))
            pos = m.end()
        spoken += say(sent[pos:])
        spans.append((start, len(spoken), MARK.sub("", sent)))
    return spoken, marks, spans


async def synth(spoken, path):
    c = edge_tts.Communicate(spoken, VOICE, rate=RATE, boundary="WordBoundary")
    words = []
    with open(path, "wb") as f:
        async for ch in c.stream():
            if ch["type"] == "audio":
                f.write(ch["data"])
            elif ch["type"] == "WordBoundary":
                words.append((ch["offset"] / 1e7, (ch["offset"] + ch["duration"]) / 1e7, ch["text"]))
    return words


def align(spoken, words):
    """Char position -> (start, end) time of each word, in order."""
    low, at, out = spoken.lower(), 0, []
    for s, e, w in words:
        i = low.find(w.lower(), at)
        if i < 0:
            continue
        out.append((i, i + len(w), s, e))
        at = i + len(w)
    return out


def time_at(aligned, pos, end=False):
    if end:
        prev = [a for a in aligned if a[0] < pos]
        return prev[-1][3] if prev else 0.0
    nxt = [a for a in aligned if a[1] > pos]
    return nxt[0][2] if nxt else aligned[-1][3]


def duration(p):
    r = subprocess.run(["ffprobe", "-v", "error", "-show_entries", "format=duration", "-of", "csv=p=0", p], capture_output=True, text=True)
    return float(r.stdout.strip())


async def main():
    only = set(sys.argv[1:])
    meta = []
    for i, sc in enumerate(S):
        if LANG == "bn":
            spoken, marks, spans = plan(" ".join(BN[i]))
            en_lines = re.split(r"(?<=[.!?])\s+", MARK.sub("", sc["text"]).strip())
            assert len(spans) == len(en_lines), (i, len(spans), len(en_lines))
        else:
            spoken, marks, spans = plan(sc["text"])
            assert len(spans) == len(sc["bn"]), (i, len(spans), len(sc["bn"]), sc["text"][:60])
        mp3 = f"{OUT}/s{i:02d}.mp3"
        cache = f"{OUT}/s{i:02d}.json"
        old = json.load(open(cache, encoding="utf-8")) if os.path.exists(cache) else None
        if old and old["spoken"] == spoken and not (only and str(i) in only):
            words = old["words"]
        else:
            for attempt in range(4):
                try:
                    words = await synth(spoken, mp3)
                    break
                except Exception as e:  # network hiccup
                    print("retry", i, e)
                    await asyncio.sleep(3)
        al = align(spoken, words)
        m = [{"key": k, "click": c, "t": time_at(al, p)} for k, c, p in marks]
        if LANG == "bn":
            sents = [{"t0": time_at(al, a), "t1": time_at(al, b, end=True), "bn": d, "en": en_lines[j]} for j, (a, b, d) in enumerate(spans)]
        else:
            sents = [{"t0": time_at(al, a), "t1": time_at(al, b, end=True), "en": d, "bn": sc["bn"][j]} for j, (a, b, d) in enumerate(spans)]
        info = {"spoken": spoken, "words": words, "marks": m, "sents": sents, "dur": duration(mp3)}
        json.dump(info, open(cache, "w", encoding="utf-8"), ensure_ascii=False)
        meta.append(info)
        print(f"s{i:02d} {info['dur']:5.1f}s  marks={[(x['key'], round(x['t'], 1)) for x in m]}")
    print("total voice", round(sum(x["dur"] for x in meta), 1), "s")


asyncio.run(main())
