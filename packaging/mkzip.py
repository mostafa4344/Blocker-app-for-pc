# Builds a zip with forward-slash paths (Chrome Web Store / Linux / macOS need them;
# Windows PowerShell 5 Compress-Archive writes backslashes).
import os, sys, zipfile
# mkzip.py <out.zip> <src_dir> [prefix]  -- forward-slash paths, deflate
out, src = sys.argv[1], sys.argv[2]
prefix = sys.argv[3] if len(sys.argv) > 3 else ""
with zipfile.ZipFile(out, "w", zipfile.ZIP_DEFLATED) as z:
    for root, _, files in os.walk(src):
        for f in sorted(files):
            full = os.path.join(root, f)
            rel = os.path.relpath(full, src).replace(os.sep, "/")
            z.write(full, (prefix + "/" + rel) if prefix else rel)
