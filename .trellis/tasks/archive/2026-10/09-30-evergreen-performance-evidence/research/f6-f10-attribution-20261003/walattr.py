"""诊断：运行 replay-facade，周期复制 WAL，按提交统计脏页所属 b-tree 对象。"""
import json, shutil, struct, subprocess, sys, time
from collections import Counter, defaultdict
from pathlib import Path

EXE = Path(r"D:\Documents\Code\Github\clash-verge-ai-residential\bench-data\t05-rebuild-20261002-4f8a8c68\executables")


def run(variant, active, workload, out):
    out = Path(out)
    if out.exists():
        shutil.rmtree(out)
    data = out / "data"
    snaps = out / "snaps"
    snaps.mkdir(parents=True)
    rev = "c278bb7b56603001e32e353d2ee589dccef0bfe9" if variant == "baseline" else "diag"
    argv = [str(EXE / f"{variant}-monitor-bench.exe"), "replay-facade", "--active", str(active), "--hz", "1", "--duration-secs", "30",
            "--warmup-secs", "5", "--workload", workload, "--metadata-change-percent", "100", "--archive", "complete",
            "--query-every-frames", "5", "--seed", "20260919", "--start-utc", "1800001800", "--source-revision", rev, "--dir", str(data)]
    fo, fe = open(out / "report.json", "wb"), open(out / "stderr.log", "wb")
    proc = subprocess.Popen(argv, stdout=fo, stderr=fe)
    n = 0
    while proc.poll() is None:
        db, wal = data / "monitor.sqlite3", data / "monitor.sqlite3-wal"
        try:
            if wal.exists() and wal.stat().st_size > 32:
                d = snaps / f"{n:04d}"
                d.mkdir()
                shutil.copyfile(wal, d / "wal")
                shutil.copyfile(db, d / "db")
                n += 1
        except OSError:
            pass
        time.sleep(0.5)
    fo.close(); fe.close()
    return proc.returncode, n


def wal_frames(raw):
    magic, ver, psize, seq, s1, s2 = struct.unpack(">IIIIII", raw[:24])
    frames, off = [], 32
    while off + 24 + psize <= len(raw):
        pg, commit, f1, f2 = struct.unpack(">IIII", raw[off:off + 16])
        if (f1, f2) != (s1, s2):
            break
        frames.append((pg, commit, raw[off + 24:off + 24 + psize]))
        off += 24 + psize
    return psize, frames


def varint(b, i):
    v = 0
    for k in range(9):
        c = b[i + k]
        if k == 8:
            return (v << 8) | c, i + 9
        v = (v << 7) | (c & 0x7F)
        if c < 0x80:
            return v, i + k + 1


def owners(pages, psize):
    """返回 page -> object 名。"""
    def page(n):
        return pages[n]

    def cells(n):
        p = page(n)
        h = 100 if n == 1 else 0
        t = p[h]
        cnt = struct.unpack(">H", p[h + 3:h + 5])[0]
        hl = 12 if t in (2, 5) else 8
        ptrs = [struct.unpack(">H", p[h + hl + 2 * i:h + hl + 2 * i + 2])[0] for i in range(cnt)]
        right = struct.unpack(">I", p[h + 8:h + 12])[0] if t in (2, 5) else None
        return t, ptrs, right

    def payload_overflow(p, off, t, plen):
        u = psize
        x = u - 35 if t == 13 else ((u - 12) * 64 // 255) - 23
        if plen <= x:
            return None
        m = ((u - 12) * 32 // 255) - 23
        k = m + (plen - m) % (u - 4)
        local = k if k <= x else m
        return struct.unpack(">I", p[off + local:off + local + 4])[0]

    own = {}

    def walk(n, name):
        own[n] = name
        t, ptrs, right = cells(n)
        p = page(n)
        for off in ptrs:
            if t in (2, 5):
                walk(struct.unpack(">I", p[off:off + 4])[0], name)
                off += 4
            if t == 5:
                continue
            plen, off = varint(p, off)
            if t == 13:
                _, off = varint(p, off)
            ov = payload_overflow(p, off, t, plen)
            while ov:
                own[ov] = name + "(overflow)"
                ov = struct.unpack(">I", page(ov)[:4])[0]
        if right:
            walk(right, name)

    # sqlite_master 记录：type,name,tbl_name,rootpage,sql
    roots = []
    def master(n):
        t, ptrs, right = cells(n)
        p = page(n)
        for off in ptrs:
            if t == 5:
                master(struct.unpack(">I", p[off:off + 4])[0]); continue
            plen, off = varint(p, off)
            _, off = varint(p, off)
            hlen, j = varint(p, off)
            types, end = [], off + hlen
            while j < end:
                st, j = varint(p, j); types.append(st)
            vals, j = [], end
            for st in types:
                if st >= 13:
                    ln = (st - 12) // 2 if st % 2 == 0 else (st - 13) // 2
                    vals.append(p[j:j + ln].decode("utf-8", "replace")); j += ln
                elif st in (1, 2, 3, 4, 6):
                    ln = {1: 1, 2: 2, 3: 3, 4: 4, 6: 8}[st]
                    vals.append(int.from_bytes(p[j:j + ln], "big", signed=True)); j += ln
                elif st == 8: vals.append(0)
                elif st == 9: vals.append(1)
                else: vals.append(None)
            if len(vals) >= 4 and isinstance(vals[3], int) and vals[3] > 0:
                roots.append((vals[1], vals[3]))
        if right:
            master(right)
    master(1)
    walk(1, "sqlite_master")
    for name, root in roots:
        walk(root, name)
    return own


def analyze(out):
    out = Path(out)
    snaps = sorted((out / "snaps").iterdir())
    best = None
    for d in reversed(snaps):
        raw = (d / "wal").read_bytes()
        psize, frames = wal_frames(raw)
        if sum(1 for f in frames if f[1]) >= 5:
            best = (d, psize, frames); break
    d, psize, frames = best
    db = (d / "db").read_bytes()
    pages = {i + 1: db[i * psize:(i + 1) * psize] for i in range(len(db) // psize)}
    commits, cur = [], []
    for pg, commit, img in frames:
        pages[pg] = img
        cur.append(pg)
        if commit:
            commits.append(cur); cur = []
    own = owners(pages, psize)
    per = []
    for c in commits:
        per.append(Counter(own.get(pg, "free/unknown") for pg in c))
    return {"snapshot": d.name, "commits": len(commits), "frames_per_commit": [len(c) for c in commits], "per_commit": [dict(x) for x in per]}


if __name__ == "__main__":
    variant, active, workload, out = sys.argv[1], int(sys.argv[2]), sys.argv[3], sys.argv[4]
    rc, n = run(variant, active, workload, out)
    result = analyze(out)
    result.update({"exit": rc, "snapshots": n})
    Path(out, "attribution.json").write_text(json.dumps(result, ensure_ascii=False, indent=1), encoding="utf-8")
    print(json.dumps({"exit": rc, "snapshots": n, "commits": result["commits"], "frames": result["frames_per_commit"][-8:]}))
