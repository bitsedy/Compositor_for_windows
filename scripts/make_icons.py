import os
import struct
import shutil

icons_dir = "crates/compositor-desktop/icons"
os.makedirs(icons_dir, exist_ok=True)

src_dir = "Compositor/Assets.xcassets/AppIcon.appiconset"
for f in os.listdir(src_dir):
    if f.endswith(".png"):
        shutil.copy(os.path.join(src_dir, f), os.path.join(icons_dir, f))

shutil.copy(os.path.join(src_dir, "app-icon-32.png"), os.path.join(icons_dir, "32x32.png"))
shutil.copy(os.path.join(src_dir, "app-icon-128.png"), os.path.join(icons_dir, "128x128.png"))
shutil.copy(os.path.join(src_dir, "app-icon-256.png"), os.path.join(icons_dir, "128x128@2x.png"))

sizes = [16, 32, 64, 128, 256]
entries_data = []
for s in sizes:
    p = os.path.join(src_dir, f"app-icon-{s}.png")
    if os.path.exists(p):
        with open(p, "rb") as f:
            data = f.read()
            entries_data.append((s, data))

header = struct.pack("<HHH", 0, 1, len(entries_data))
offset = 6 + 16 * len(entries_data)
dir_entries = b""
payload = b""
for s, data in entries_data:
    w = s if s < 256 else 0
    h = s if s < 256 else 0
    dir_entries += struct.pack("<BBBBHHII", w, h, 0, 0, 1, 32, len(data), offset)
    payload += data
    offset += len(data)

with open(os.path.join(icons_dir, "icon.ico"), "wb") as f:
    f.write(header + dir_entries + payload)

print("Icons generated successfully:", len(entries_data))
