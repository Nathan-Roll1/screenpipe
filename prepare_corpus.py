"""Prepare existing Screenpipe fixtures without changing their references."""
import hashlib
import json
from pathlib import Path
import subprocess
import sys

root = Path(__file__).parent.resolve()
source = Path(sys.argv[1]).resolve()
rows = json.loads((root / "screenpipe-corpus.json").read_text())
(root / "audio").mkdir(exist_ok=True)
chapter = root / "repo-fixtures/screenpipe/bench-quality"
chapter.mkdir(parents=True, exist_ok=True)
references = []
for row in rows:
    original = source / "crates/screenpipe-audio" / row["source_path"]
    if hashlib.sha256(original.read_bytes()).hexdigest() != row["source_sha256"]:
        raise ValueError(f"Fixture hash mismatch: {original}; fetch its Git LFS payload")
    wav = root / row["path"]
    subprocess.run([
        "ffmpeg", "-y", "-v", "error", "-i", str(original),
        "-ac", "1", "-ar", "16000", "-c:a", "pcm_s16le", str(wav),
    ], check=True)
    subprocess.run([
        "ffmpeg", "-y", "-v", "error", "-i", str(wav),
        str(chapter / (row["id"] + ".flac")),
    ], check=True)
    references.append(row["id"] + " " + row["reference"])
(chapter / "screenpipe-bench-quality.trans.txt").write_text("\n".join(references) + "\n")
print(f"Prepared {len(rows)} Screenpipe fixtures in {root}")
