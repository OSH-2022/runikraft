#![no_std]

extern crate alloc;

use core::ptr::null;
use rkalloc::RKalloc;
use rksched::RKsched;
use rkschedbasis::RKthreadAttr;
use runikraft::list::Tailq;
use crate::blkdev_core::{RkBlkdev, RkBlkdevData, RkBlkdevEventHandler, RkBlkdevQueueEventT};

mod blkdev;
mod blkdev_core;
mod blkdev_driver;
mod blkreq;

static mut RK_BLKDEV_LIST:Tailq<RkBlkdev>=Tailq;
static mut BLKDEV_COUNT: Option<i16> = None;
const CONFIG_LIBUKBLKDEV_MAXNBQUEUES: u16 =  core::u16::from_str(env!("PATH"));

pub unsafe fn _alloc_data<'a>(a: &'a (dyn RKalloc + 'a), blkdev_id: u16, drv_name: &'a str) -> *mut RkBlkdevData<'a> {
    //TODO let mut data: *mut RkBlkdevData = alloc_type::<RkBlkdevData>(a, ());
    //这仅仅会发生在我们设置设备身份的时候
    //在设备生命的剩余时间，这个身份是只读的
    todo!()
}

#[cfg(feature = "dispatcherthreads")]
pub fn _dispatcher(args:*mut u8) {
    let handler=RkBlkdevEventHandler;
    loop{
        //TODO uk_semaphore_down(&handler->events);
        handler.callback(handler.dev,handler.queue_id,handler,cookie);
    }
}


#[cfg(not(feature = "dispatcherthreads"))]
pub fn _create_event_handler(callback:RkBlkdevQueueEventT, cookie:*mut u8, event_handler: &mut RkBlkdevEventHandler) ->isize {
    event_handler.callback=callback;
    event_handler.cookie=cookie;
    0
}

#[cfg(feature = "dispatcherthreads")]
pub fn _create_event_handler(callback:RkBlkdevQueueEventT, cookie:*mut u8, dev:*RkBlkdev, queue_id:u16, s:*mut RKsched, event_handler: &mut RkBlkdevEventHandler) ->isize {
    event_handler.callback=callback;
    event_handler.cookie=cookie;
    //如果我们没有回调，我们就不需要线程
    if callback.is_null(){
        return 0;
    }
    event_handler.dev=dev;
    event_handler.queue_id=queue_id;
    //TODO uk_semaphore_init(&event_handler->events, 0);
    event_handler.dispatcher_s=s;
    //为分派器线程创造一个名字
    //如果有错误，我们就在没有名字的状况下继续
    //TODO if (asprintf(&event_handler->dispatcher_name,
    // 		     "blkdev%" PRIu16 "-q%" PRIu16 "]", dev->_data->id,
    // 		     queue_id)
    // 	    < 0) {
    // 		event_handler->dispatcher_name = NULL;
    // 	}
    //创建线程
    unsafe{event_handler.dispatcher=(*event_handler.dispatcher_s).thread_create(event_handler.dispatcher_name, &mut RKthreadAttr::default(), _dispatcher, event_handler as *mut u8);}
    if event_handler.dispatcher.is_null() {
        if event_handler.dispatcher_name.is_null() {
            
        }
    }
    0
}

#[cfg(feature = "dispatcherthreads")]
pub fn _destory_event_handler() {
    todo!()
}

pub fn ptriseer(ptr: i64) -> bool {
    if ptr <= 0 && ptr >= -512 {
        true
    } else {
        false
    }
}