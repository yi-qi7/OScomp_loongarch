//ext4
//use easy_fs::BlockDevice; 
//ext4下面三个
use alloc::vec;
use alloc::vec::Vec;
use ext4_rs::{BlockDevice, BLOCK_SIZE};

//ext4
/* use isomorphic_drivers::{
    block::ahci::{AHCI, BLOCK_SIZE},
    provider,
}; */
//ext4
use isomorphic_drivers::{
    block::ahci::{AHCI},
    provider,
};
//ext4
const DEVICE_BLOCK_SIZE: usize = 512;

use log::info;

use crate::{
    config::PAGE_SIZE,
    loongarch::VIRT_BIAS,
    mm::{frame_alloc, frame_dealloc, PhysAddr},
    phys_to_virt, println,
    sync::UPSafeCell,
    virt_to_phys,
};

pub struct AHCIDriver(UPSafeCell<AHCI<Provider>>);

impl AHCIDriver {
    pub fn new(header: usize, size: usize) -> Option<Self> {
        unsafe { AHCI::new(header, size).map(|x| Self(UPSafeCell::new(x))) }
    }
}

impl BlockDevice for AHCIDriver {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        log::error!("read_block");
        self.0.exclusive_access().read_block(block_id, buf);
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        assert!(buf.len() >= BLOCK_SIZE);
        self.0.exclusive_access().write_block(block_id, buf);
    }

    fn read_offset(&self, offset: usize) -> Vec<u8> {
        log::error!("read_offset");
        //起始块id
        let block_id = offset / DEVICE_BLOCK_SIZE;
        //结果缓冲区
        let mut result_buf = vec![0u8; BLOCK_SIZE];
        let inner_offset = offset % DEVICE_BLOCK_SIZE;
        log::error!("offset read: inner_offset={}", inner_offset);
        //无偏移
        if inner_offset == 0 {
            self.read_block(block_id, &mut result_buf);
        } 
        else { //有偏移
            log::error!("4096+512");
            //let mut temp_buf = vec![0u8; BLOCK_SIZE + DEVICE_BLOCK_SIZE]; //这里是4096+512
            let mut temp_buf4096 = vec![0u8; BLOCK_SIZE]; //4096
            let mut temp_buf512 = vec![0u8; BLOCK_SIZE]; //512
            // 读取第一个块(4096字节)
            self.read_block(block_id, &mut temp_buf4096);
            
            // 需要读取第二个块(4096字节)
            self.read_block(block_id + 1, &mut temp_buf512); //块id+1
            
            // 从两个缓冲区中组合数据
            // 数据跨越两个块
            let first_part_len = DEVICE_BLOCK_SIZE - inner_offset;
            let second_part_len = 4096 - first_part_len;
            
            // 复制第一部分(来自4096字节块)
            result_buf[0..first_part_len].copy_from_slice(
                &temp_buf4096[inner_offset..DEVICE_BLOCK_SIZE]);
            
            // 复制第二部分(来自512字节块)
            result_buf[first_part_len..BLOCK_SIZE].copy_from_slice(
                &temp_buf512[0..second_part_len]);

            log::error!("good");
        }
        // println!("result_buf: {:x}", result_buf.as_ptr() as usize);
        println!("bye read_offset");

        result_buf
    }

    //ext4
    fn write_offset(&self, offset: usize, data: &[u8]) {
        let start_block_id = offset / DEVICE_BLOCK_SIZE;
        let write_len = data.len();
        let end_block_id = (offset + write_len - 1) / DEVICE_BLOCK_SIZE;
        let fit_len = (end_block_id - start_block_id + 1) * DEVICE_BLOCK_SIZE;
        
        // println!("write_offset: start_block_id: {}, write_len: {}, offset: {}, data_addr: {:x}, fit_len: {}, temp buf len {}", start_block_id, write_len, offset, data.as_ptr() as usize, fit_len, temp_buf.len());
        //采用两次io，添加一层判断，如果temp_buf超了4096就分成两部分进行两次io
        if fit_len > 4096{
            // 考虑迭代处理，这样可以应对两次io无法解决的情况
            // 每段的最大长度
            let segment_size = 4096;
            
            let mut current_offset = offset;
            let mut current_start_block = start_block_id;
            let mut remaining_data = data;
            
            while !remaining_data.is_empty() {
                // 计算当前段的长度
                let current_len = remaining_data.len().min(segment_size);
                let current_end_block = (current_offset + current_len - 1) / DEVICE_BLOCK_SIZE;
                let current_fit_len = (current_end_block - current_start_block + 1) * DEVICE_BLOCK_SIZE;
                // 这是不超过4096的temp_buf
                let mut temp_buf = vec![0u8; current_fit_len];
                
                // 读取当前段的数据
                self.read_block(current_start_block, &mut temp_buf);
                
                // 复制数据到临时缓冲区
                let buffer_offset = current_offset % DEVICE_BLOCK_SIZE;
                temp_buf[buffer_offset..buffer_offset + current_len].copy_from_slice(remaining_data);
                
                // 写回当前段的数据
                self.write_block(current_start_block, &temp_buf);
                
                // 更新偏移量和剩余数据
                current_offset += current_len;
                current_start_block = current_offset / DEVICE_BLOCK_SIZE;
                remaining_data = &remaining_data[current_len..];
            }
        }
        else{
            //原始逻辑不动
            let mut temp_buf = vec![0u8; fit_len];
            self.read_block(start_block_id, &mut temp_buf);
            // println!("write_offset: start_block_id: {}, write_len: {}, offset: {}, data_addr: {:x}, fit_len: {}", start_block_id, write_len, offset, data.as_ptr() as usize, fit_len);
            temp_buf[offset % DEVICE_BLOCK_SIZE..offset % DEVICE_BLOCK_SIZE + write_len].copy_from_slice(&data);
            
            self.write_block(start_block_id, &temp_buf);
        }
        
        
    }

    fn handle_irq(&self) {
        self.0.exclusive_access().handle_interrupt();
    }
}

struct Provider;

impl provider::Provider for Provider {
    const PAGE_SIZE: usize = PAGE_SIZE;
    fn alloc_dma(size: usize) -> (usize, usize) {
        let pages = size / PAGE_SIZE;
        let mut phy_base = 0;
        for i in 0..pages {
            let frame = frame_alloc().unwrap();
            let frame_pa: PhysAddr = frame.ppn.into();
            let frame_pa = frame_pa.into();
            core::mem::forget(frame);
            if i == 0 {
                phy_base = frame_pa;
            }
            assert_eq!(frame_pa, phy_base + i * PAGE_SIZE);
        }
        let base_page: usize = phy_base / PAGE_SIZE;
        let virt_base = phys_to_virt!(phy_base);
        println!(
            "virtio_dma_alloc: phy_addr: ({:#x}-{:#x})",
            phy_base,
            phy_base + size
        );
        (virt_base, phy_base)
    }

    fn dealloc_dma(va: usize, size: usize) {
        println!("dealloc_dma: virt_addr: ({:#x}-{:#x})", va, va + size);
        let mut pa = virt_to_phys!(va);
        let pages = size / PAGE_SIZE;
        for _ in 0..pages {
            frame_dealloc(PhysAddr::from(pa).into());
            pa += PAGE_SIZE;
        }
    }
}
