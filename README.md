# luomctl

Linux CLI to change the buttons of the LUOM G10 / Combaterwing mouse (USB `04d9:a09f`).
The mouse stores the settings, so no background service is necessary.

## Install

```
cargo build --release
sudo install -m644 70-luom-g10.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Use

```
luomctl show a                 # Mode A buttons
luomctl set 9 forward a        # button 9 sends M5
luomctl set 8 combo:08:07 b    # button 8 sends Win+D in Mode B
```

Button numbers are the numbers in the official software picture.

## Protocol

Recovered with IDA from `Combaterwing GM.exe` and checked against a USB capture.

| Function | Address | Role |
|---|---|---|
| `hid_handshake` | `0x42B760` | sends the hello, plain then encrypted |
| `hid_send_cmd` | `0x42B920` | 7 bytes + checksum, encrypts, SetFeature |
| `cmd_encrypt` | `0x42B3C0` | byte swap, XOR key, rotate 64 bits left by 3, add table |
| `cmd_decrypt` | `0x42B580` | the inverse |
| `apply_settings` | `0x411BF0` | builds the button tables |

- Transport: interface 2. Commands go as 8-byte feature reports. Bulk data goes as 32-byte reports on EP 0x04 (OUT) and 0x83 (IN).
- Command: `op, args..., checksum` where `checksum = 0xFF - sum(bytes 0..7)`. Bit 7 of `op` means read.
- Cipher key: `00 '4' '5' '2' 'A' 'E' 'A' 00`. Add table: `"RoNgtEng"`, each byte nibble-swapped.
- Hello: `00 aa dd 3b 62 00 00 db`. The mouse echoes it only when it is encrypted.
- `0x12 <table> 0x40` writes, `0x92 <table>` reads 16 button slots of 4 bytes. Table 1 is Mode A, table 3 is Mode B.
- Other opcodes seen in the capture: `0d` report rate, `07` light effect, `10` DPI colors, `11` DPI levels, `13` macros.

Slot values:

| Action | Bytes |
|---|---|
| left / right / middle | `01 00 f0 00` / `f1` / `f2` |
| back / forward | `01 00 f3 00` / `01 00 f4 00` |
| wheel up / down | `01 00 f7 00` / `01 00 f8 00` |
| DPI loop | `07 00 03 00` |
| RGB on/off | `08 00 03 00` |
| key | `00 00 <hid> 00` |
| combo | `00 <mod> <hid> 00` |
| fire key | `0a ...` |
| macro | `09 ...` |
| off | `00 00 00 00` |

`tools-cipher.py` decodes a usbmon text capture: `python3 tools-cipher.py usbmon.txt`.
