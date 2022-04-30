#[macro_use]
pub mod console;
pub mod time;
pub mod bootstrap;
pub mod lcpu;
pub mod irq;
pub mod spinlock;
//pub mod memory;
//
//pub mod thread;


// supervisor binary interface
mod sbi;

// 寄存器
mod reg;