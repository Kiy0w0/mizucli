"""Extract Bad Apple frames from video.mp4 → src/bad_apple_frames.rs (RLE encoded)."""
import cv2
import sys

VIDEO = "src/data/video.mp4"
OUT = "src/bad_apple_frames.rs"
W, H = 80, 45  # frame size (chars)
THRESHOLD = 128  # pixel >= threshold → 1 (white), else 0 (black)

cap = cv2.VideoCapture(VIDEO)
if not cap.isOpened():
    print(f"ERROR: cannot open {VIDEO}", file=sys.stderr)
    sys.exit(1)

total = int(cap.get(cv2.CAP_PROP_FRAME_COUNT))
print(f"Extracting {total} frames at {W}x{H}...")

def rle_encode(pixels):
    runs = []
    i = 0
    while i < len(pixels):
        val = pixels[i]
        count = 1
        while i + count < len(pixels) and pixels[i + count] == val and count < 255:
            count += 1
        runs.append((count, val))
        i += count
    return runs

frames = []
idx = 0
while True:
    ret, frame = cap.read()
    if not ret:
        break
    gray = cv2.cvtColor(frame, cv2.COLOR_BGR2GRAY)
    small = cv2.resize(gray, (W, H), interpolation=cv2.INTER_AREA)
    pixels = [1 if p >= THRESHOLD else 0 for p in small.flatten()]
    frames.append(rle_encode(pixels))
    idx += 1
    if idx % 500 == 0:
        print(f"  {idx}/{total}")

cap.release()
print(f"  {idx}/{total} done. Writing {OUT}...")

with open(OUT, "w", encoding="utf-8") as f:
    f.write(f"pub const W: u16 = {W};\n")
    f.write(f"pub const H: u16 = {H};\n")
    f.write(f"pub const FRAME_COUNT: usize = {len(frames)};\n\n")
    f.write("pub static FRAMES: &[&[(u8, u8)]] = &[\n")
    for i, rle in enumerate(frames):
        pairs = ", ".join(f"({c},{v})" for c, v in rle)
        f.write(f"    &[{pairs}],\n")
    f.write("];\n")

print(f"Done. {len(frames)} frames written.")
