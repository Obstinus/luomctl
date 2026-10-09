# luomctl

Linux CLI to change the buttons of the LUOM G10 / Combaterwing mouse (USB `04d9:a09f`).
The mouse stores the settings, so no background service is necessary.

## Install

```
cargo build --release
sudo install -m644 70-luom-g10.rules /etc/udev/rules.d/
sudo udevadm control --reload-rules && sudo udevadm trigger
```

## Window

Start **Mouse buttons** from the app menu, or run `luom-mouse`.
Choose an action for each button. Then click **Save to mouse**.
The **Lights** tab sets the light effect. The **Response rate** tab sets the report rate.

Install for one user:

```
cargo install --path . --root ~/.local
install -Dm644 luom-mouse.desktop ~/.local/share/applications/
```

The window draws the mouse with the button numbers. The numbers are the same as in the official software.

## Command line

```
luomctl show a                 # Mode A buttons
luomctl set 9 forward a        # button 9 sends M5
luomctl set 8 combo:08:07 b    # button 8 sends Win+D in Mode B
luomctl rate                   # show the report rate
luomctl rate 500               # set the report rate to 500 Hz
luomctl light                  # show the light effect
luomctl light steady           # set the steady colour
```

Button numbers are the numbers in the official software picture.
Rates: `125`, `250`, `500`, `1000` (Hz). Lights: `breathing`, `flashing`, `steady`.

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
- Other opcodes seen in the capture: `01` report rate (`81` read), `0d` light mode (`8d` read), `07` light slots (`87` read), `10` DPI colors, `11` DPI levels, `13` macros.
- Light writes go in this order: slot 0 (`07 00`), slot 1 (`07 01`), mode (`0d`). The tool waits 2 ms between frames.

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

Report rate (opcode `01`, read `81`, the reply byte 1 holds the code):

| Rate | Code |
|---|---|
| 1000 Hz | `01` |
| 500 Hz | `02` |
| 250 Hz | `04` |
| 125 Hz | `08` |

Light presets (mode `0d` holds kind, param, option. Slot 0 is `07 00`):

| CLI name | Preset | Kind | Option | Slot 0 |
|---|---|---|---|---|
| `breathing` | Breathing, colour cycle | `02` | `02` | `01 fa 02` |
| `flashing` | Breathing, flashing | `02` | `03` | `02 02 03` |
| `steady` | Steady colour | `03` | kept | `03 05 05` |

The param byte stays while the kind stays. After a kind change it is `05` for kind `02` and `01` for kind `03`. Slot 1 is always written back unchanged.

`tools-cipher.py` decodes a usbmon text capture: `python3 tools-cipher.py usbmon.txt`.
