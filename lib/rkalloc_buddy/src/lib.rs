// SPDX-License-Identifier: BSD-3-Clause
// rkallocbuddy/lib.rs

// Authors: 张子辰 <zichen350@gmail.com>

// Copyright (C) 2022 吴骏东, 张子辰, 蓝俊玮, 郭耸霄 and 陈建绿.

// Redistribution and use in source and binary forms, with or without
// modification, are permitted provided that the following conditions
// are met:
// 1. Redistributions of source code must retain the above copyright
//    notice, this list of conditions and the following disclaimer.
// 2. Redistributions in binary form must reproduce the above copyright
//    notice, this list of conditions and the following disclaimer in the
//    documentation and/or other materials provided with the distribution.
// 3. Neither the name of the copyright holder nor the names of its
//    contributors may be used to endorse or promote products derived from
//    this software without specific prior written permission.
// THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
// AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
// IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
// ARE DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE
// LIABLE FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR
// CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF
// SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS
// INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN
// CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
// ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
// POSSIBILITY OF SUCH DAMAGE.

//! 伙伴分配器（buddy allocator）。
//! 
//! # 设计
//! 
//! 这里不介绍伙伴分配器的定义，只介绍具体的实现。
//! 
//! 首先考虑大小为2^n的被管理的内存区域。
//! 
//! 最小的块的长度是2^4 bytes，能容纳2个指针；最大的块的大小为2^48 bytes，是AMD64支持的最大内存容量。
//! 用双向链表维护空闲块，链表的结点`Node`储存在它对应的内存区域的开头。
//! 用树状bitset维护所有的内存区块的分配情况（元数据）：
//! - `[0]`是根结点；
//! - `[i*2+1]`是`[i]`的左孩子，它对应的内存区域是`i`二分后的前半段；
//! - `[i*2+2]`是`[i]`的右孩子，它对应的内存区域是`i`二分后的后半段；
//! - 顺序（order）为`k`的结点在这个bitset的索引范围是`[2^(n-k)-1:2^(n-k+1)-2]`；
//! - 顺序为`k`的结点共有2^(n-k)个，每个的大小为2^k
//! - 整个bitset的大小为2^(n+1)/16/8=2^(n-6) bytes
//! - bitset[i] = 0 表示结点i：
//!     - i 没有被分配
//!     - i 的父结点已经被分配
//! - bitset[i] = 1 表示结点i：
//!     - i 被分配
//!     - i 被二分成了两个子结点
//! 初始时，元数据的所有位都是0，如果一块内存i被分配，则i的孩子一定全是0，i和i的祖先一定全是1。
//! 
//! 元数据被储存在被管理的内存区块的末尾，它们需要占用2^(n-10)个最小的块。
//! 
//! 当内存区域的大小size不是2的幂时，记n=ceil(log2(size))，则可以将内存区域视为大小为2^n但末尾的一些
//! 结点已经被分配的内存区域。
#![no_std]

use rkalloc::{RKalloc};
use core::cmp::max;
use core::ptr::null_mut;

//最小的内存块大小
const MIN_SIZE: usize = 1usize << MIN_ORDER;
//最大的内存块大小
const MAX_SIZE: usize = 1usize << MAX_ORDER;
const MIN_ORDER: usize = 4;
const MAX_ORDER: usize = 48;
//页的对齐要求
const PAGE_ALIGNMENT: usize = 4096;

pub struct RKallocBuddy<'a> {
    //空闲区块列表(双向循环链表)，order-MIN_ORDER才是free_list_head的索引
    //【注意】访问free_list_head时，下标通常是[i-MIN_ORDER]
    free_list_head: [*mut Node; MAX_ORDER - MIN_ORDER + 1],
    max_order: usize,   //order的最大值，等于floor(log2(total))
    root_order: usize,  //根区块的order，等于ceil(log2(total))
    meta_data: Bitset<'a>,
    base: *const Node,  //内存空间的基地址，只被index函数使用，为了方便，干脆用Node指针
    //统计信息
    size_left: usize,   //剩余可用空间大小
    size_total: usize,  //总可用空间大小
}

/// 大小和Node相同的Bitset, 储存空间的分配情况
struct Bitset<'a> {
    data: &'a mut [usize]
}

impl Bitset<'_> {
    unsafe fn new(data: *mut usize, len: usize) -> Self {
        data.write_bytes(0, len);
        Bitset{data: core::slice::from_raw_parts_mut(data, len)}
    }
    fn get(&self, index: usize) -> bool {
        (self.data[index/64] & (1usize<<index%64)) !=0
    }
    fn set(&mut self, index: usize, data: bool) {
        if data {
            self.data[index/64] |= 1usize<<index%64;
        }
        else {
            self.data[index/64] &= !(1usize<<index%64);
        }
    }
}

#[inline(always)] fn lchild(i: usize)->usize {i*2+1}
#[inline(always)] fn rchild(i: usize)->usize {i*2+2}
#[inline(always)] fn parent(i: usize)->usize {(i-1)/2}
#[inline(always)] fn sibling(i: usize)->usize {((i-1)^1)+1}

#[derive(Clone, Copy)]
struct Node {
    pub pre: *mut Node,     //前驱结点
    pub next: *mut Node,    //后继结点
}

impl Node {
    ///初始化循环双链表（pre和next都指向自己）
    fn init(&mut self) {
        self.pre = self;
        self.next = self;
    }
}

///在结点head之后插入结点node
unsafe fn insert_node(head: *mut Node, node: *mut Node) {
    debug_assert!(!head.is_null());
    debug_assert!(!node.is_null());
    (*node).next = (*head).next;
    (*head).next = node;
    debug_assert!(!(*node).next.is_null());
    (*(*node).next).pre = node;
    (*node).pre = head;
}

///把一个结点移出链表
unsafe fn remove_node(node: *mut Node){
    debug_assert!(!node.is_null());
    debug_assert!(!(*node).pre.is_null());
    debug_assert!(!(*node).next.is_null());
    debug_assert!((*node).next != (*node).pre);
    (*(*node).pre).next=(*node).next;
    (*(*node).next).pre=(*node).pre;
}

const fn log2_usize(mut x: usize) -> usize{
    let mut y = 0_usize;
    if x >= 4294967296 {y+=32; x>>=32;}
    if x >= 65536 {y+=16; x>>=16;}
    if x >= 256 {y+=8; x>>=8;}
    if x >= 16 {y+=4; x>>=4;}
    if x >= 4 {y+=2; x>>=2;}
    if x >=2 {y+=1;}
    y
}

impl RKallocBuddy<'_> {
    ///确定一个结点的在meta_data中的索引
    #[inline(always)]
    fn index(&self, addr: *const Node, order: usize) -> usize {
        debug_assert!(order>=MIN_ORDER);
        debug_assert!(order<=self.max_order);
        debug_assert!(addr>=self.base);
        (1<<(self.root_order-order)) - 1 + unsafe{addr.offset_from(self.base) as usize}
    }

    /// 创建伙伴分配器示例
    /// - `base`: 内存区域的基地址，必须4k对齐
    /// - `size`: 内存区域的大小，不必是2^n，但必须是16的倍数
    /// # 安全性
    /// - base..base+size范围的地址不能有其他用途
    pub unsafe fn new(base: *mut u8, size: usize)->Self {
        debug_assert!(!base.is_null());
        debug_assert!(size % MIN_SIZE == 0);
        debug_assert!(base as usize % PAGE_ALIGNMENT == 0);
        debug_assert!(size <= MAX_SIZE);

        //总的16B-块数
        let n_blocks = size/MIN_SIZE;
        //用来存元数据的16B-块数
        let mut n_meta_blocks = (n_blocks+64)/65; //ceil(n_blocks/65)
        //用来存能被分配出去的数据的16B-块数
        let mut n_data_blocks = n_blocks - n_meta_blocks;
        if !n_data_blocks.is_power_of_two() {
            n_data_blocks -= n_meta_blocks;
            n_meta_blocks *= 2;
        }
        debug_assert!(n_meta_blocks >= n_data_blocks/64);
        //let meta_size = n_meta_blocks*MIN_SIZE;
        let data_size = n_data_blocks*MIN_SIZE;

        let max_order = log2_usize(data_size);
        let root_order = if 1<<max_order == data_size {max_order} else{max_order+1};
        let mut free_list_head = [null_mut(); MAX_ORDER - MIN_ORDER + 1];
        debug_assert!((1<<root_order-MIN_ORDER+1)/64 < n_meta_blocks*2);

        //将空闲结点加入空闲结点链表
        {
            let mut size = data_size;
            let mut base = base;
            while size > 0{
                let i = log2_usize(size);
                let node = base as *mut Node;
                (*node).init();
                free_list_head[i-MIN_ORDER] = node;
                base = base.offset(1<<i);
                size -= 1<<i;
            }
        }
        
        RKallocBuddy {
            free_list_head,
            max_order,
            root_order,
            meta_data: Bitset::new(base.add(size) as *mut usize, n_meta_blocks*2),
            base: base as *const Node,
            size_left: data_size,
            size_total: size,
        }
    }

    /// 这里的size就是实际上要分配的结点的大小，由buddy allocator的特点，size正确就一定能满足对其要求
    /// 调用前要对self加锁
    unsafe fn alloc_mut(&mut self, size: usize) -> *mut u8 {
        let log2size = log2_usize(size);
        let mut i = log2size;
        //从log2size开始，找到大小为2^i的空闲的块
        while i<=self.max_order && self.free_list_head[i-MIN_ORDER].is_null(){
            i+=1;
        }
        //找不到大小足够的块
        if i > self.max_order {return null_mut();}

        let ptr = self.free_list_head[i-MIN_ORDER];
        debug_assert!(!ptr.is_null());
        if (*ptr).next != (*ptr).pre {remove_node(ptr);}
        else {self.free_list_head[i-MIN_ORDER]=null_mut();}

        while i!=log2size {
            i-=1;
            self.split(ptr, i);
        }
        
        // 清空元数据
        (*ptr).pre = null_mut();
        (*ptr).next = null_mut();
        // 更新统计信息
        self.size_left -= size;
        ptr as *mut u8
    }

    unsafe fn dealloc_mut(&mut self, ptr: *mut u8, size: usize) {
        let mut ptr = ptr as *mut Node;
        let mut order = log2_usize(size);
        let mut i = self.index(ptr, order);
        //i的伙伴是空闲的，将i与i的伙伴合并
        while i!=0 && !self.meta_data.get(sibling(i)) && self.meta_data.get(parent(i)){
            ptr = self.merge(ptr, order, i);
            order += 1;
            i = parent(i);
        }
        if !self.free_list_head[order-MIN_ORDER].is_null(){
            insert_node(self.free_list_head[order-MIN_ORDER], ptr);
        }
        else {
            (*ptr).init();
            self.free_list_head[order-MIN_ORDER] = ptr;
        }
        self.size_left += size;
    }

    /// 将一个大的内存块分割成两个小的，将地址较大的一段插入free_list_head[order-MIN_ORDER]处
    /// 【注意】order是ptr拆分后的order，而不是ptr本身的order
    unsafe fn split(&mut self, mut ptr: *mut Node, order: usize){
        let i = self.index(ptr, order+1);
        //把它标记为被分裂了
        debug_assert!(self.meta_data.get(i)==false);
        self.meta_data.set(i,true); 
        //它的分裂产生的两个子结点应该处在可用状态
        debug_assert!(self.meta_data.get(lchild(i))==false);
        debug_assert!(self.meta_data.get(rchild(i))==false);
        ptr = (ptr as *mut u8).offset(1<<order) as *mut Node; //现在ptr指向分裂后的结点的右孩子
        //找不到更小的块才会去尝试拆分更大的块, 所以order层的可用结点列表一定是空的
        debug_assert!(self.free_list_head[order-MIN_ORDER].is_null());
        (*ptr).init();
        self.free_list_head[order-MIN_ORDER] = ptr;
    }

    /// 把位于`free_list`之外的`ptr`与位于`free_list`之内的伙伴内存块合并，返回合并后的内存块的地址
    /// 合并后的内存块**不**被拆入`free_list`中，`order`是ptr自身的order, `i`是`ptr`在meta_data中的索引
    unsafe fn merge(&mut self, ptr: *mut Node, order: usize, i: usize) -> *mut Node{
        debug_assert_eq!(i, self.index(ptr, order));
        //i左孩子，i的伙伴的起始地址是ptr+(1<<order-MIN_ORDER)
        if i%2 == 1 {
            let buddy = ptr.add(1<<order-MIN_ORDER);
            if (*buddy).pre != (*buddy).next {remove_node(buddy);}
            else {self.free_list_head[order-MIN_ORDER] = null_mut();}
            debug_assert!(self.meta_data.get(parent(i)));
            self.meta_data.set(parent(i), false);
            ptr
        }
        else {
            let buddy = ptr.sub(1<<order-MIN_ORDER);
            if (*buddy).pre != (*buddy).next {remove_node(buddy);}
            else {self.free_list_head[order-MIN_ORDER] = null_mut();}
            debug_assert!(self.meta_data.get(parent(i)));
            self.meta_data.set(parent(i), false);
            buddy
        }
    }
}

unsafe impl RKalloc for RKallocBuddy<'_> {
    unsafe fn alloc(&self, size: usize, align: usize) -> *mut u8{
        debug_assert!(align.is_power_of_two());
        debug_assert!(align<=PAGE_ALIGNMENT);
        //实际上需要分配的内存大小
        let size = max(max(size,align),MIN_SIZE);
        //剩余空间不足
        if self.size_left < size{
            return null_mut();
        }
        let mut_self = &mut *(self as *const Self as *mut Self);
        return mut_self.alloc_mut(size);
    }
    unsafe fn dealloc(&self, ptr: *mut u8, size: usize, align: usize) {
        debug_assert!(align.is_power_of_two());
        debug_assert!(align<=PAGE_ALIGNMENT);
        let mut_self = &mut *(self as *const Self as *mut Self);
        let size = max(max(size,align),MIN_SIZE);
        mut_self.dealloc_mut(ptr, size);
    }
}

// unsafe impl RKallocExt for RKallocBuddy<'_> {
    
// }
