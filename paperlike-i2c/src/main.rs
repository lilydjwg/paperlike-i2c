use std::time::Duration;
use std::thread::sleep;
use std::os::unix::io::{RawFd, AsRawFd};
use std::fs::OpenOptions;
use std::io;

fn main() {
  while let Err(e) = run() {
    eprintln!("Error: {:?}", e);
    sleep(Duration::from_secs(10));
  }
}

fn run() -> io::Result<()> {
  let file = OpenOptions::new()
    .read(true).write(true)
    .open("/dev/i2c-1")?;
  let fd = file.as_raw_fd();

  let msg1 = [81, 132, 3, 8, 1, 1, 176];
  let msg2 = [110, 136, 2, 0, 8, 0, 0, 1, 0, 2, 191];
  loop {
    send(fd, 0x37, &msg1, 0)?;
    sleep(Duration::from_millis(100));
    send(fd, 0x37, &msg2, 1)?;
    sleep(Duration::from_secs(3));
  }

  // debug code
  //
  // use std::fs::File;
  // use std::io::{BufRead, BufReader};
  //
  // let file = File::open("out").unwrap();
  // let file = BufReader::new(file);
  // for line in file.lines() {
  //   let line = line.unwrap();
  //   let parts: Vec<&str> = line.split(' ').collect();
  //   let delay = parts[1].parse().unwrap();
  //   let addr = u16::from_str_radix(parts[3], 16).unwrap();
  //   let flags = parts[5].parse().unwrap();
  //   let msg: Vec<u8> = parts[9..].iter().map(|x| x.parse().unwrap()).collect();
  //   let ret = send(fd, addr, &msg, flags);
  //   println!("sent {:?} to {:x} ret {}", msg, addr, ret);
  //   assert!(ret == 1);
  //   sleep(Duration::from_millis(delay));
  // }
}

#[repr(C)]
struct Transfer {
  msgs: *const Message,
  nmsgs: u32,
}

#[repr(C)]
struct Message {
  addr: u16,
  flags: u16,
  len: u16,
  buf: *const u8,
}

fn send(fd: RawFd, addr: u16, msg: &[u8], flags: u16) -> io::Result<()> {
  let message = Message {
    addr,
    flags,
    len: msg.len() as u16,
    buf: msg.as_ptr(),
  };
  let msgs = [message];
  let transfer = Transfer {
    msgs: msgs.as_ptr(),
    nmsgs: 1,
  };
  let ret = unsafe {
    libc::ioctl(fd, 0x707, &transfer as *const _)
  };

  if ret < 0 {
    Err(io::Error::last_os_error())
  } else {
    Ok(())
  }
}

