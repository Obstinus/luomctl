//! hidraw access to the vendor interface (interface 2) of the mouse.

use crate::cipher::{decrypt, encrypt};
use crate::{LightState, ReportRate};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::os::fd::AsRawFd;
use std::path::PathBuf;
use std::time::Duration;

const HID_ID: &str = "HID_ID=0003:000004D9:0000A09F";
/// Report descriptor of the vendor interface starts with Usage Page 0xFF00.
const VENDOR_PAGE: [u8; 3] = [0x06, 0x00, 0xff];
const HELLO: [u8; 8] = [0x00, 0xaa, 0xdd, 0x3b, 0x62, 0x00, 0x00, 0xdb];

/// One table of 16 button slots, 4 bytes each.
pub type ButtonTable = [[u8; 4]; 16];

pub struct Mouse {
    file: File,
}

fn ioc(dir: u32, nr: u32, size: u32) -> libc::c_ulong {
    ((dir << 30) | (size << 16) | ((b'H' as u32) << 8) | nr) as libc::c_ulong
}

fn find_hidraw() -> io::Result<PathBuf> {
    for entry in fs::read_dir("/sys/class/hidraw")? {
        let dir = entry?.path();
        let uevent = fs::read_to_string(dir.join("device/uevent")).unwrap_or_default();
        let desc = fs::read(dir.join("device/report_descriptor")).unwrap_or_default();
        if uevent.contains(HID_ID) && desc.starts_with(&VENDOR_PAGE) {
            return Ok(PathBuf::from("/dev").join(dir.file_name().unwrap()));
        }
    }
    Err(io::Error::new(io::ErrorKind::NotFound, "mouse 04d9:a09f not found"))
}

fn command(bytes: &[u8]) -> [u8; 8] {
    let mut p = [0u8; 8];
    p[..bytes.len()].copy_from_slice(bytes);
    p[7] = 0xffu8.wrapping_sub(p[..7].iter().fold(0u8, |a, b| a.wrapping_add(*b)));
    p
}

impl Mouse {
    pub fn open() -> io::Result<Self> {
        let path = find_hidraw()?;
        let file = OpenOptions::new().read(true).write(true).open(&path).map_err(|e| {
            io::Error::new(e.kind(), format!("{}: {e} (install the udev rule)", path.display()))
        })?;
        let mut mouse = Mouse { file };
        mouse.handshake()?;
        Ok(mouse)
    }

    fn set_feature(&self, data: [u8; 8]) -> io::Result<()> {
        let mut buf = [0u8; 9];
        buf[1..].copy_from_slice(&data);
        let r = unsafe { libc::ioctl(self.file.as_raw_fd(), ioc(3, 6, 9), buf.as_mut_ptr()) };
        if r < 0 { Err(io::Error::last_os_error()) } else { Ok(()) }
    }

    fn get_feature(&self) -> io::Result<[u8; 8]> {
        let mut buf = [0u8; 9];
        let r = unsafe { libc::ioctl(self.file.as_raw_fd(), ioc(3, 7, 9), buf.as_mut_ptr()) };
        if r < 0 {
            return Err(io::Error::last_os_error());
        }
        Ok(buf[1..].try_into().unwrap())
    }

    /// Same flow as `hid_handshake` (0x42B760): the mouse echoes the hello only when it is encrypted.
    fn handshake(&mut self) -> io::Result<()> {
        self.set_feature(HELLO)?;
        std::thread::sleep(Duration::from_millis(5));
        self.get_feature()?;
        self.set_feature(encrypt(HELLO))?;
        std::thread::sleep(Duration::from_millis(5));
        if decrypt(self.get_feature()?)[1..5] != HELLO[1..5] {
            return Err(io::Error::other("handshake failed: the mouse did not answer"));
        }
        Ok(())
    }

    fn send(&self, bytes: &[u8]) -> io::Result<[u8; 8]> {
        self.set_feature(encrypt(command(bytes)))?;
        if bytes[0] & 0x80 != 0 {
            return Ok(decrypt(self.get_feature()?));
        }
        Ok([0; 8])
    }

    fn read_report(&mut self) -> io::Result<[u8; 32]> {
        let mut pfd = libc::pollfd { fd: self.file.as_raw_fd(), events: libc::POLLIN, revents: 0 };
        if unsafe { libc::poll(&mut pfd, 1, 1000) } <= 0 {
            return Err(io::Error::new(io::ErrorKind::TimedOut, "no data from the mouse"));
        }
        let mut buf = [0u8; 32];
        self.file.read_exact(&mut buf)?;
        Ok(buf)
    }

    /// Opcode 0x92: read the 64-byte button table of a mode (1 = Mode A, 3 = Mode B).
    pub fn read_buttons(&mut self, table: u8) -> io::Result<ButtonTable> {
        let reply = self.send(&[0x92, table])?;
        if reply[..3] != [0x92, table, 0x40] {
            return Err(io::Error::other(format!("unexpected reply {reply:02x?}")));
        }
        let mut out = [[0u8; 4]; 16];
        for half in 0..2 {
            let report = self.read_report()?;
            for i in 0..8 {
                out[half * 8 + i].copy_from_slice(&report[i * 4..i * 4 + 4]);
            }
        }
        Ok(out)
    }

    /// Opcode 0x12: write the 64-byte button table of a mode, as two 32-byte output reports.
    pub fn write_buttons(&mut self, table: u8, slots: &ButtonTable) -> io::Result<()> {
        self.send(&[0x12, table, 0x40])?;
        for half in slots.chunks(8) {
            let mut report = [0u8; 33];
            for (i, slot) in half.iter().enumerate() {
                report[1 + i * 4..5 + i * 4].copy_from_slice(slot);
            }
            self.file.write_all(&report)?;
        }
        Ok(())
    }

    /// Reads a command reply. Byte 0 echoes the opcode.
    fn read_reply(&self, op: &[u8]) -> io::Result<[u8; 8]> {
        let reply = self.send(op)?;
        if reply[0] != op[0] {
            return Err(io::Error::other(format!("unexpected reply {reply:02x?}")));
        }
        Ok(reply)
    }

    /// Opcode 0x81: read the report rate code (reply byte 1).
    pub fn read_rate(&self) -> io::Result<ReportRate> {
        let code = self.read_reply(&[0x81])?[1];
        ReportRate::from_byte(code).ok_or_else(|| io::Error::other(format!("unknown report rate code {code:02x}")))
    }

    /// Opcode 0x01: write the report rate code.
    pub fn write_rate(&self, rate: ReportRate) -> io::Result<()> {
        self.send(&[0x01, rate.to_byte()]).map(|_| ())
    }

    /// Opcodes 0x8d (mode) and 0x87 00 / 0x87 01 (slots): read the light registers.
    pub fn read_light(&self) -> io::Result<LightState> {
        Ok(LightState {
            mode: three(self.read_reply(&[0x8d])?, 1),
            slot0: three(self.read_reply(&[0x87, 0x00])?, 2),
            slot1: three(self.read_reply(&[0x87, 0x01])?, 2),
        })
    }

    /// Opcodes 0x07 00 and 0x07 01 (slots), then 0x0d (mode). The mode goes last, as in the official software.
    pub fn write_light(&self, state: &LightState) -> io::Result<()> {
        let pause = Duration::from_millis(2);
        let [s0, s1, s2] = state.slot0;
        self.send(&[0x07, 0x00, s0, s1, s2])?;
        std::thread::sleep(pause);
        let [s0, s1, s2] = state.slot1;
        self.send(&[0x07, 0x01, s0, s1, s2])?;
        std::thread::sleep(pause);
        let [m0, m1, m2] = state.mode;
        self.send(&[0x0d, m0, m1, m2]).map(|_| ())
    }
}

/// Three value bytes of a light reply, starting at `start`.
fn three(reply: [u8; 8], start: usize) -> [u8; 3] {
    [reply[start], reply[start + 1], reply[start + 2]]
}
