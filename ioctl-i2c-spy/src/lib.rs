use std::sync::atomic::{Ordering, AtomicIsize};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;

static FD: AtomicIsize = AtomicIsize::new(0);
const TARGET_FILE: &[u8] = b"/dev/i2c-5";

fn timestamp_ms() -> u64 {
  use std::time::{SystemTime, UNIX_EPOCH};
  let now = SystemTime::now();
  let elapsed = now.duration_since(UNIX_EPOCH).unwrap();
  elapsed.subsec_millis() as u64
}

redhook::hook! {
  unsafe fn open64(
    path: *const c_char,
    flags: i32,
    mode: i32
  ) -> i32 => fileopen64 {
    let ret = redhook::real!(open64)(
      path, flags, mode);
    let path = CStr::from_ptr(path).to_bytes();
    if path == TARGET_FILE {
      FD.store(ret as isize, Ordering::SeqCst);
    }
    if let Ok(p) = std::str::from_utf8(path) {
      println!("open64 {} as {}", p, ret);
    }
    return ret;
  }
}

redhook::hook! {
  unsafe fn open(
    path: *const c_char,
    flags: i32,
    mode: i32
  ) -> i32 => fileopen {
    let ret = redhook::real!(open)(
      path, flags, mode);
    let path = CStr::from_ptr(path).to_bytes();
    if path == TARGET_FILE {
      FD.store(ret as isize, Ordering::SeqCst);
    }
    if let Ok(p) = std::str::from_utf8(path) {
      println!("open {} as {}", p, ret);
    }
    return ret;
  }
}

redhook::hook! {
  unsafe fn __open_2(
    path: *const c_char,
    flags: i32
  ) -> i32 => file_open_2 {
    let ret = redhook::real!(__open_2)(
      path, flags);
    let path = CStr::from_ptr(path).to_bytes();
    if path == TARGET_FILE {
      FD.store(ret as isize, Ordering::SeqCst);
    }
    if let Ok(p) = std::str::from_utf8(path) {
      println!("__open_2 {} as {}", p, ret);
    }
    return ret;
  }
}

redhook::hook! {
  unsafe fn ioctl(
    fd: i32,
    request: u64,
    data: *const u8
  ) -> i32 => ioctlspy {
    let target_fd = FD.load(Ordering::SeqCst) as i32;
    if fd > 0 && fd == target_fd && request == 0x707 {
      let transfer = &*(data as *const Transfer);
      println!("req: {:x} {} msgs", request, transfer.nmsgs);
      let msgs = slice::from_raw_parts(transfer.msgs, transfer.nmsgs as usize);
      let t = timestamp_ms();
      for msg in msgs {
        let data: &[u8] = slice::from_raw_parts(msg.buf, msg.len as usize);
        println!("msg: ts {} addr {:x} flags {:x} len {} data: {:?}",
          t, msg.addr, msg.flags, msg.len, data);
      }
    }

    redhook::real!(ioctl)(fd, request, data)
  }
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
