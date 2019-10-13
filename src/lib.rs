use byteorder::{ByteOrder, LittleEndian, WriteBytesExt};
use mozlz4_sys::*;


const MAGIC_NUMBER: &[u8] = b"mozLz40\0";

pub fn decompress(ibuffer: Vec<u8>) -> Result<Vec<u8>, String> {
    let magic_number_len = MAGIC_NUMBER.len();
    if ibuffer.len() < (magic_number_len + 4) || !ibuffer.starts_with(MAGIC_NUMBER) {
        return Err("Unrecognized input file".into())
    }

    let decompressed_size = ibuffer
        .get(magic_number_len..(magic_number_len + 4))
        .unwrap();
    let decompressed_size = LittleEndian::read_u32(decompressed_size) as usize;

    let block = ibuffer.get(magic_number_len + 4..).unwrap();
    let mut obuffer: Vec<u8> = Vec::with_capacity(decompressed_size);

    unsafe {
        let bytes_decompressed = LZ4_decompress_safe(
            block.as_ptr() as *const i8,
            obuffer.as_mut_ptr() as *mut i8,
            block.len() as i32,
            decompressed_size as i32,
        );
        if bytes_decompressed < 0 {
            return Err("Malformed input file".into())
        }
        obuffer.set_len(bytes_decompressed as usize);
    }
    Ok(obuffer)
}

pub fn compress(ibuffer: Vec<u8>) -> Result<Vec<u8>, String> {
    let uncompressed_size = ibuffer.len();
    let compress_bound = unsafe { LZ4_compressBound(uncompressed_size as i32) as usize };

    let magic_number_len = MAGIC_NUMBER.len();
    let mut obuffer: Vec<u8> = Vec::with_capacity(magic_number_len + 4 + compress_bound);
    obuffer.extend(MAGIC_NUMBER);
    obuffer
        .write_u32::<LittleEndian>(uncompressed_size as u32)
        .unwrap();

    unsafe {
        let bytes_compressed = LZ4_compress_default(
            ibuffer.as_ptr() as *const i8,
            obuffer[(magic_number_len + 4)..].as_mut_ptr() as *mut i8,
            uncompressed_size as i32,
            compress_bound as i32,
        );
        if bytes_compressed <= 0 {
            return Err("Compression failed".into())
        }
        obuffer.set_len(magic_number_len + 4 + bytes_compressed as usize);
    }
    Ok(obuffer)
}
