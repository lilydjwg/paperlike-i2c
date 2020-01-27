use std::sync::atomic::{Ordering, AtomicIsize};
use std::ffi::CStr;
use std::os::raw::c_char;
use std::slice;

static FD: AtomicIsize = AtomicIsize::new(0);

redhook::hook! {
  unsafe fn open64(
    path: *const c_char,
    flags: i32,
    mode: i32
  ) -> i32 => fileopen64 {
    let ret = redhook::real!(open64)(
      path, flags, mode);
    let path = CStr::from_ptr(path).to_bytes();
    if path == b"/dev/i2c-1" {
      FD.store(ret as isize, Ordering::SeqCst);
    }
    if let Ok(p) = std::str::from_utf8(path) {
      println!("opening {} as {}", p, ret);
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
    if path == b"/dev/i2c-1" {
      FD.store(ret as isize, Ordering::SeqCst);
    }
    if let Ok(p) = std::str::from_utf8(path) {
      println!("opening {} as {}", p, ret);
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
    if fd > 0 && fd == target_fd {
      let transfer = &*(data as *const Transfer);
      println!("req: {:x} {} msgs", request, transfer.nmsgs);
      let msgs = slice::from_raw_parts(transfer.msgs, transfer.nmsgs as usize);
      let dur = time::PrimitiveDateTime::now() - time::PrimitiveDateTime::unix_epoch();
      let t = dur.whole_milliseconds();
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
