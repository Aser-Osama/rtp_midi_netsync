// Test FFI, will be replaced with a proper FFI layer later.
use crate::header::PayloadHeader;
use std::ffi::c_int;
use std::os::raw::c_uchar;

#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct FFIPayloadHeader {
    pub flags: c_uchar,
    pub len: c_uchar,
}

#[no_mangle]
pub extern "C" fn FFI_parse_header(buf: *const c_uchar, buf_len: c_int) -> FFIPayloadHeader {
    // Safety: caller must ensure buf is valid for `buf_len` bytes
    let slice = unsafe {
        if buf.is_null() || buf_len <= 0 {
            return FFIPayloadHeader { flags: 0, len: 0 };
        }
        std::slice::from_raw_parts(buf, buf_len as usize)
    };

    match PayloadHeader::parse(slice) {
        Ok(parsed) => FFIPayloadHeader {
            flags: parsed.flags,
            len: parsed.len,
        },
        Err(_) => FFIPayloadHeader { flags: 0, len: 0 },
    }
}

#[no_mangle]
pub extern "C" fn FFI_serialize_header(hdr: FFIPayloadHeader, out_buf: *mut c_uchar) {
    let output = PayloadHeader {
        flags: hdr.flags,
        len: hdr.len,
    }
    .serialize();

    // Safety: caller must provide at least 1 byte of writeable memory
    unsafe {
        if !out_buf.is_null() {
            *out_buf = output[0];
        }
    }
}
