#![no_std]

use rkschedbasis::{SchedulerCoop, RKthread, RKthreadAttr};
use runikraft::list::Tailq;
use rkalloc::RKalloc;
use core::time::Duration;

pub struct RKschedcoop<'a> {
    threads_started: bool,
    idle: RKthread<'a>,
    exited_threads: Tailq<'a, RKthread<'a>>,
    // plat_ctx_cbs: /* plat context callbacks 类型*/
    allocator: &'a dyn RKalloc,
    next: &'a mut RKsched<'a>,
    prv: *mut u8,
}

impl<'a> RKschedcoop<'a> {
    pub fn new() -> Self {
        todo!()
    }
}

impl<'b> SchedulerCoop for RKschedcoop<'b> {
    fn yield_sched(&mut self) {
        todo!()
    }
    fn add_thread<'a>(&mut self, t: &'a mut RKthread<'a>, attr: &'a mut RKthreadAttr) {
        todo!()
    }
    fn remove_thread<'a>(&mut self, t: &'a mut RKthread<'a>) {
        todo!()
    }
    fn block_thread<'a>(&mut self, t: &'a mut RKthread<'a>) {
        todo!()
    }
    fn wake_thread<'a>(&mut self, t: &'a mut RKthread<'a>) {
        todo!()
    }
    fn sleep_thread(&self, nsec: Duration) {
        todo!()
    }
    fn exit_thread(&self) {
        todo!()
    }
}