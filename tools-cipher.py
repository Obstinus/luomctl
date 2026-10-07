KEY = bytes([0, ord('4'), ord('5'), ord('2'), ord('A'), ord('E'), ord('A'), 0])
TBL = b"RoNgtEng"
ADD = [((t >> 4) + (t << 4)) & 0xFF for t in TBL]

def enc(p):
    r = bytearray(8)
    r[0], r[5], r[1], r[4], r[2], r[7], r[3], r[6] = p[5], p[0], p[4], p[1], p[7], p[2], p[6], p[3]
    for i in range(1, 7): r[i] ^= KEY[i]
    v = int.from_bytes(r, "big"); v = ((v << 3) | (v >> 61)) & (2**64 - 1)
    r = bytearray(v.to_bytes(8, "big"))
    return bytes((r[i] + ADD[i]) & 0xFF for i in range(8))

def dec(c):
    r = bytearray((c[i] - ADD[i]) & 0xFF for i in range(8))
    v = int.from_bytes(r, "big"); v = ((v >> 3) | (v << 61)) & (2**64 - 1)
    r = bytearray(v.to_bytes(8, "big"))
    for i in range(1, 7): r[i] ^= KEY[i]
    p = bytearray(8)
    p[5], p[0], p[4], p[1], p[7], p[2], p[6], p[3] = r[0], r[5], r[1], r[4], r[2], r[7], r[3], r[6]
    return bytes(p)

if __name__ == "__main__":
    import sys, re
    hello = bytes.fromhex("00aadd3b620000db")
    print("enc(hello) =", enc(hello).hex(), "(capture: 27ad550da17fbc5e)")
    for line in open(sys.argv[1]):
        m = re.search(r" S Co:\S+ s 21 09 0300 0002 0008 8 = (\w{8}) (\w{8})", line)
        if m:
            c = bytes.fromhex(m[1] + m[2]); p = dec(c)
            ok = (0xFF - sum(p[:7])) & 0xFF == p[7]
            print(c.hex(), "->", p.hex(), "sum-ok" if ok else "")
