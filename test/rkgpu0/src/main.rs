// test rkgpu

#![no_std]
#![no_main]

extern crate rkboot;
extern crate rkgpu;

use rkgpu::*;
// use core::slice;
// use core::mem::{size_of, align_of};
// use core::ptr::NonNull;

#[no_mangle]
unsafe fn main(_args: &mut [&str])->i32 {
    init();
    rkplat::println!("\nTest rkgpu0 passed!\n");
    loop{}
    return 0;
}