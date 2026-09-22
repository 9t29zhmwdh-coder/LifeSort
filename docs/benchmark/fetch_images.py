"""Downloads the benchmark photos listed in manifest.tsv from Wikimedia Commons.

The images are not part of the repository; each keeps the licence named in
the manifest. They are fetched at 1600 px width, the size messenger apps
typically send.

    python3 fetch_images.py <target-dir>
"""
import csv, os, sys, time, urllib.parse, urllib.request

UA = {"User-Agent": "LifeSortBench/1.0 (https://github.com/9t29zhmwdh-coder/LifeSort)"}
target = sys.argv[1] if len(sys.argv) > 1 else "images"
os.makedirs(target, exist_ok=True)
here = os.path.dirname(os.path.abspath(__file__))
with open(os.path.join(here, "manifest.tsv"), newline="", encoding="utf-8") as f:
    rows = list(csv.DictReader(f, delimiter="\t"))
fetched = []
for row in rows:
    url = "https://commons.wikimedia.org/wiki/Special:FilePath/" + urllib.parse.quote(row["title"]) + "?width=1600"
    data = None
    for attempt in range(4):
        try:
            data = urllib.request.urlopen(urllib.request.Request(url, headers=UA), timeout=60).read()
            break
        except OSError:
            time.sleep(5 * (attempt + 1))  # Commons throttles with HTTP 429
    # Write only what arrived: an empty file would count as a photo nobody can decode.
    if data:
        with open(os.path.join(target, row["file"]), "wb") as out:
            out.write(data)
        fetched.append(row)
    else:
        print(f"skipped {row['file']}: not available after 4 attempts", file=sys.stderr)
    time.sleep(1)
rows = fetched
with open(os.path.join(target, "manifest.tsv"), "w", encoding="utf-8") as out:
    out.write("file\tlabel\n" + "".join(f"{r['file']}\t{r['label']}\n" for r in rows))
print(f"{len(rows)} images in {target}")
