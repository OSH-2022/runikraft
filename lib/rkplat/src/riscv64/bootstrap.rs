//TODO: arg

use core::ptr::{null_mut, addr_of, addr_of_mut};
use core::{slice, arch};
use runikraft::config::rkplat::*;
use runikraft::config::STACK_SIZE_SCALE as SSS;
use crate::drivers::device_tree;

use super::sbi::*;

extern "C" {
    /// 在初始化时有平台层调用
    /// 
    /// 非平台层的库必须提供其实现
    /// 
    /// 在拥有已分析的参数的平台，这个函数会被直接调用；否则，平台层将调用
    /// `rkplat_entry_argp`，并由它分析参数，然后调用`rkplat_entry`
    /// - `argc`: 参数个数
    /// - `argv`: 参数，每个参数是NTBS
    pub fn rkplat_entry(argc: i32, argv: *mut *mut u8) -> !;

    /// 在初始化时有平台层调用
    /// 
    /// 非平台层的库必须提供其实现
    /// 
    /// 在没有已分析的参数的平台，平台层将调用
    /// `rkplat_entry_argp`，由它分析参数，然后调用`rkplat_entry`
    /// - `arg0`: NTBS，参数0，即镜像的名称；可能为空，这时分析后的参数的argv[0]也需要留空
    /// - `argb`: 剩余的参数
    /// - `argb_len`: 剩余的参数的长度，`argb_len=0`表示`argb`是空终止的
    pub fn rkplat_entry_argp(arg0: *mut u8, argb: *mut u8, argb_len: usize) -> !;

    fn __rkplat_newstack(stack_top: *mut u8, tramp: extern fn(*mut u8)->!, arg: *mut u8)->!;
}

#[no_mangle] extern "C" fn __runikraft_entry_point2(_arg: *mut u8) -> !{
    unsafe{
        rkplat_entry(0,0 as *mut *mut u8);
    }
}

const DEVICE_TREE_MAGIC: u32 = 0xD00DFEED;

#[repr(C)]
#[derive(Debug,Clone,Copy)]
pub(crate) struct HartLocal {
    _reg_space: usize,  //供中断处理程序使用的临时保存寄存器的空间(offset 0)
    hartsp: usize,          //异常处理程序使用的栈的指针    (offset 8)
    hartid: usize,          // offset 16
}

impl HartLocal {
    const fn new()->Self {
        Self { _reg_space: 0, hartid: 0, hartsp: 0}
    }
}

/// 读取内核本地数据
pub(crate) unsafe fn hart_local() -> &'static mut HartLocal{
    let mut scratch: usize;
    arch::asm!("csrr {scratch}, sscratch",
        scratch=out(reg)(scratch));
    (scratch as *mut HartLocal).as_mut().unwrap()
}

pub(crate) static mut HART_NUMBER: usize = 0;
pub(crate) static mut HART_LOCAL:[HartLocal;LCPU_MAXCOUNT] = [HartLocal::new();LCPU_MAXCOUNT];
static mut EXCEPT_STACK:[[usize;128*SSS];LCPU_MAXCOUNT] = [[0;128*SSS];LCPU_MAXCOUNT];
static mut MAIN_STACK:[usize;MAIN_STACK_SIZE/8*SSS] = [0;MAIN_STACK_SIZE/8*SSS];


#[repr(C)]
struct DeviceTreeHeader {
    be_magic: u32,
    be_size: u32,
}

//debug: addi    sp,sp,-560
//release: addi    sp,sp,-112
#[no_mangle]
unsafe fn __runikraft_entry_point(hartid: usize, device_ptr: usize) -> !{
    //在OpenSBI下，编号最大的hart会继续执行到Supervisor mode, 而其他harts会被暂停，所以，此时的hartid+1就是hart数
    HART_NUMBER = hartid+1;
    for i in 0..=hartid {
        HART_LOCAL[i].hartid = i;
        HART_LOCAL[i].hartsp = (addr_of!(EXCEPT_STACK[i]) as usize)+1024;
    }
    let scratch_addr = addr_of!(HART_LOCAL[hartid]);
        arch::asm!("csrw sscratch, {s}",
            s=in(reg)scratch_addr);
    let header = &*(device_ptr as *const DeviceTreeHeader);
    let magic = u32::from_be(header.be_magic);
    assert_eq!(magic,DEVICE_TREE_MAGIC);
    let len = u32::from_be(header.be_size) as usize;
    let buffer = slice::from_raw_parts(device_ptr as *const u8, len);
    match device_tree::parse(buffer) {
        Ok(_) => {},
        Err(e) => {panic!("Fail to load device tree. {:?}",&e);}
    }
    __rkplat_newstack((addr_of_mut!(MAIN_STACK) as *mut u8).add(MAIN_STACK_SIZE), __runikraft_entry_point2,null_mut());
}

/// 退出
pub fn halt() -> ! {
    sbi_call(SBI_SRST, 0, 0, 0, 0).unwrap();
    panic!("Should halt.");
}

/// 重启
pub fn restart() -> ! {
    sbi_call(SBI_SRST, 0, 1, 0, 0).unwrap();
    panic!("Should restart.");
}

/// 崩溃
pub fn crash() -> ! {
    print!("System crashed!\n");
    sbi_call(SBI_SRST, 0, 0, 1, 0).unwrap();
    loop {}//不能用panic，因为panic会调用crash
}

/// 挂起
pub fn suspend() -> ! {
    todo!();
}
