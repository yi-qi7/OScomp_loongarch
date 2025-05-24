#![no_std]
#![no_main]
#![feature(alloc_error_handler)]
#![allow(unused)]
#![feature(naked_functions)]
mod boot;
mod config;
mod fs;
mod info;
mod lang_items;
mod logging;
mod loongarch;
mod mm;
mod print;
mod sync;
mod syscall;
mod task;
mod timer;
mod trap;
mod uart;
extern crate alloc;

use core::arch::global_asm;

use config::{FLAG, UART};
pub use log::info;
use uart::Uart;

use crate::{
    fs::list_apps,
    info::{kernel_layout, print_machine_info},
    loongarch::{
        ahci_init, extioi_init, i8042_init, ls7a_intc_init, rtc_init, rtc_time_read, vbe_test,
    },
    task::add_initproc,
    timer::get_time_ms,
    trap::enable_timer_interrupt,
};

// global_asm!(include_str!("head.s"));
// sbss 和 ebss来自连接脚本,此处定义为 extern "C"
extern "C" {
    fn sbss();
    fn ebss();
}

pub fn clear_bss() {
    unsafe {
        core::slice::from_raw_parts_mut(
            sbss as usize as *mut u128,
            (ebss as usize - sbss as usize) / size_of::<u128>(),  //潜在问题，若BSS段字节数不是16的整数倍，末尾部分字节不会被初始化
        ) 
        .fill(0);
        
    }
}


/*测试内存是否可写
//test_memory_write(sbss as *mut u64);
fn test_memory_write(address: *mut u64) {
    unsafe {
        // 尝试写入并读取回显值
        core::ptr::write_volatile(address, 0x12345678);
        let value = core::ptr::read_volatile(address);
        if value != 0x12345678 {
            panic!("Memory write failed at {:#x}", address as usize);
        }
        println!("Memory test passed at {:#x}", address as usize);
    }
}*/



#[no_mangle]
pub fn main(cpu: usize) {
    println!("Hello, world!{}",cpu);
    println!("{}", FLAG);
    println!("cpu: {}", cpu);
    clear_bss();
    println!("{}", FLAG);
    println!("cpu: {}", cpu); //输出为0,这是因为龙芯架构中主核ID固定为0
    logging::init();
    log::error!("Logging init success");
    rtc_init();
    println!("CURRENT TIME {:?}", rtc_time_read());
    kernel_layout();

    mm::init();
    if cfg!(feature = "gui") {
        // 外部中断控制器初始化
        extioi_init();
        println!("extioi init success");
        // 桥片中断初始化
        ls7a_intc_init();
        println!("ls7a intc init success");
        // 键盘
        i8042_init();
        println!("i8042 init success");
        // gui
    }

    trap::init();
    print_machine_info();
    println!("machine info success");
    // sata硬盘
    ahci_init();
    println!("ahci init success");
    //运行程序

    if cfg!(feature = "gui") {
        vbe_test();
    }

    enable_timer_interrupt();

    // list_apps(); //列出所有程序
    add_initproc(); //添加初始化程序
    println!("add initproc success");

    task::run_tasks(); //运行程序
    panic!("main end");
}
