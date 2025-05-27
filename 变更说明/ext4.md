# 文件系统迁移为ext4

**所有更改均伴随“ext4”的注释**

## 更改的文件

复制ext4_rs文件到根目录

复制virtio-drivers到根目录

在根目录创建img文件

修改Makefile文件

修改kernel/Cargo.toml文件

os/src/fs/inode.rs -> kernel/src/fs/inode.rs

os/src/drivers/block/mod.rs -> kernel/src/loongarch/driver/mod.rs代码更改

os/src/drivers/block/virtio_blk.rs -> kernel/src/loongarch/driver/ahci.rs代码更改。

## 详细更改
仅记录与ext4不同的地方

### Makefile
由于文件结构不同，Makefile下的img路径有更改
```rust
#文件模拟块设备
# FS_IMG := ./target/$(TARGET)/$(MODE)/fs.img #ext4
FS_IMG := ./img/ex4.img #ext4 由于makefile位置要改为在当前文件夹下
```
不使用virtio-blk-device设备而是使用STATA硬盘模拟，并添加了Ahci协议。
```rust
ifeq ($(BOARD),qemu)
	qemu-system-loongarch64 \
		-m 1G \
		-smp 1 \
		-kernel $(KERNEL_ELF) \
		$(VGA) \
		-drive file=$(FS_IMG),if=none,format=raw,id=x0 \
		-device ahci,id=ahci0 \  #延用ahci协议，此处不变
		-device ide-hd,drive=x0,bus=ahci0.0  
endif
```

---

### kernel/Cargo.toml
不使用virtio-blk-device设备而是使用STATA硬盘模拟，并添加了Ahci协议。
```rust
isomorphic_drivers = { path = "../isomorphic_drivers" } #使用
#virtio-drivers = { path = "../virtio-drivers" }  不使用virtio-drivers
```

---

### kernel/src/fs/inode.rs
龙芯gui中实现了Inode下的一个函数用于获取根目录下的文件列表，并在桌面上显示文件图标。risc无这个函数，将代码替换为ext4适配的时候需加上这个函数，不然编译报错
```rust
pub fn ls(&self) -> Vec<String> {
	// 具体实现在sasy-fs/src/vfs.rs中，但迁移有问题
	vec![] // 返回空列表
}
```
该函数具体实现参考sasy-fs/src/vfs.rs，但如果将该函数迁移过来，又会涉及到更多东西，需要把更多东西一起迁移过来……

这里为了简便直接返回空列表，这仅会影响桌面显示文件图标的功能，不会对其他造成影响

将所有UPIntrFreeCell改为PSafeCell，这是因为龙芯中未实现UPIntrFreeCell，曾尝试在kernel/src/sync/up.rs中添加上UPIntrFreeCell，具体的添加过程在[最后的废案中](https://github.com/yi-qi7/OScomp_loongarch/blob/main/%E5%8F%98%E6%9B%B4%E8%AF%B4%E6%98%8E/ext4.md#%E5%BC%95%E5%85%A5upintrfreecell%E5%BA%9F%E6%A1%88%E5%87%BA%E7%8E%B0%E6%9B%B4%E5%A4%9A%E6%8A%A5%E9%94%99)，但添加完成后发现出现更多报错，需要迁移更改更多文件，所以暂时使用龙芯之前使用的PSafeCell

所以不用加上
```rust
use crate::sync::UPIntrFreeCell;
```

---

### kernel/src/loongarch/driver/mod.rs
使用ahci协议
```rust
mod ahci;
pub mod pci;
```

保留原代码对BLOCK_DEVICE的处理
```rust
//ext4原代码
/*lazy_static! {
    pub static ref BLOCK_DEVICE: Arc<dyn BlockDevice> = Arc::new(BlockDeviceImpl::new());
}*/

//改为下面
/// Used only for initialization hacks.
pub const DUMMY_BLOCK_DEVICE: *const dyn BlockDevice =
    unsafe { transmute(&0 as *const _ as *const ahci::AHCIDriver as *const dyn BlockDevice) };

pub static BLOCK_DEVICE: Cell<Arc<dyn BlockDevice>> = unsafe { transmute(DUMMY_BLOCK_DEVICE) };

pub fn ahci_init() {
    unsafe {
        (BLOCK_DEVICE.get() as *mut Arc<dyn BlockDevice>).write(Arc::new(pci_init().unwrap()));
    }
}
```

---

### kernel/src/loongarch/driver/ahci.rs
**该文件下有大量更改**

ext4给了BLOCK_SIZE，所以将isomorphic_drivers内的BLOCK_SIZE删去，否则二次定义
```rust
use isomorphic_drivers::{
    block::ahci::{AHCI}, //更改
    provider,
};
```

采用原来实现的read_block和write_block，引入与ext4有关的read_offset和write_offset，从头写一个中断handle_irq
```rust
impl BlockDevice for AHCIDriver {
    fn read_block(&self, block_id: usize, buf: &mut [u8]) {
        self.0.exclusive_access().read_block(block_id, buf);
    }

    fn write_block(&self, block_id: usize, buf: &[u8]) {
        assert!(buf.len() >= BLOCK_SIZE);
        self.0.exclusive_access().write_block(block_id, buf);
    }

    fn read_offset(&self, offset: usize) -> Vec<u8> {
        let block_id = offset / DEVICE_BLOCK_SIZE;
        let mut result_buf = vec![0u8; BLOCK_SIZE];
        let inner_offset = offset % DEVICE_BLOCK_SIZE;
        if inner_offset == 0 {
            self.read_block(block_id, &mut result_buf);
        } else {
            let mut temp_buf = vec![0u8; BLOCK_SIZE + DEVICE_BLOCK_SIZE];
            self.read_block(block_id, &mut temp_buf);
            result_buf[..BLOCK_SIZE].copy_from_slice(
                &temp_buf[inner_offset..inner_offset + BLOCK_SIZE],);
        }
        // println!("result_buf: {:x}", result_buf.as_ptr() as usize);
        result_buf
    }

    fn write_offset(&self, offset: usize, data: &[u8]) {
        let start_block_id = offset / DEVICE_BLOCK_SIZE;
        let write_len = data.len();
        let end_block_id = (offset + write_len - 1) / DEVICE_BLOCK_SIZE;
        let fit_len = (end_block_id - start_block_id + 1) * DEVICE_BLOCK_SIZE;
        let mut temp_buf = vec![0u8; fit_len];
        // println!("write_offset: start_block_id: {}, write_len: {}, offset: {}, data_addr: {:x}, fit_len: {}, temp buf len {}", start_block_id, write_len, offset, data.as_ptr() as usize, fit_len, temp_buf.len());
        self.read_block(start_block_id, &mut temp_buf);
        // println!("write_offset: start_block_id: {}, write_len: {}, offset: {}, data_addr: {:x}, fit_len: {}", start_block_id, write_len, offset, data.as_ptr() as usize, fit_len);
        temp_buf[offset % DEVICE_BLOCK_SIZE..offset % DEVICE_BLOCK_SIZE + write_len].copy_from_slice(&data);
        
        self.write_block(start_block_id, &temp_buf);
        
    }

    fn handle_irq(&self) {
        self.0.exclusive_access().handle_interrupt();
    }
}
```
由于加入了read_block和write_block，所以要在ext4_rs/src/ext4_defs/block.rs中关于BlockDevice的trait加上这两个函数，具体更改见[后面](https://github.com/yi-qi7/OScomp_loongarch/blob/main/%E5%8F%98%E6%9B%B4%E8%AF%B4%E6%98%8E/ext4.md#ext4_rssrcext4_defsblockrs)

在isomorphic_drivers/src/block/ahci.rs实现了handle_interrupt中断，详见[后面](https://github.com/yi-qi7/OScomp_loongarch/blob/main/%E5%8F%98%E6%9B%B4%E8%AF%B4%E6%98%8E/ext4.md#isomorphic_driverssrcblockahcirs)

除此之外，保留原本的AHCIDriver的结构体形式和new()
```rust
pub struct AHCIDriver(UPSafeCell<AHCI<Provider>>);

impl AHCIDriver {
    pub fn new(header: usize, size: usize) -> Option<Self> {
        unsafe { AHCI::new(header, size).map(|x| Self(UPSafeCell::new(x))) }
    }
}
```

根据[龙芯手册](https://godones.github.io/rCoreloongArch/stat.html)我们还需要保留硬盘的读取相应的接口
```rust
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
```


---
### ext4_rs/src/ext4_defs/block.rs
在trait中加入read_block和write_block
```rust
pub trait BlockDevice: Send + Sync + Any {
    fn read_offset(&self, offset: usize) -> Vec<u8>; 
    fn write_offset(&self, offset: usize, data: &[u8]); 
    fn handle_irq(&self);
    ///Read data form block to buffer
    fn read_block(&self, block_id: usize, buf: &mut [u8]); //ext4
    ///Write data from buffer to block
    fn write_block(&self, block_id: usize, buf: &[u8]); //ext4
}
```

---
### isomorphic_drivers/src/block/ahci.rs
原有的ahci无中断实现，所以需要我们实现
```rust
impl<P: Provider> AHCI<P> {
    /// 处理 AHCI 中断
    pub fn handle_interrupt(&mut self) {
        // 读取全局中断状态
        let global_is = self.ghc.interrupt_status.read();
        
        // 检查是否有端口中断
        if global_is != 0 {
            // 清除全局中断标志
            self.ghc.interrupt_status.write(global_is);
            
            // 处理每个端口的中断
            for port_num in 0..self.ghc.num_ports() {
                if !self.ghc.has_port(port_num) {
                    continue;
                }
                
                let port = unsafe { &mut *self.ghc.port_ptr(port_num) };
                self.handle_port_interrupt(port);
            }
        }
    }
    
    /// 处理单个端口的中断
    fn handle_port_interrupt(&mut self, port: &mut AHCIPort) {
        // 读取端口中断状态
        let port_is = port.interrupt_status.read();
        
        // 检查是否有命令完成中断
        if port_is & (1 << 31) != 0 {
            // 清除端口中断标志
            port.interrupt_status.write(1 << 31);
            
            // 处理完成的命令槽
            for slot in 0..32 {
                if (port_is & (1 << slot)) != 0 {
                    self.process_completed_command(port, slot);
                }
            }
        }
    }
    
    /// 处理完成的命令
    fn process_completed_command(&mut self, port: &mut AHCIPort, slot: usize) {
        // 检查命令是否完成
        if port.command_issue.read() & (1 << slot) == 0 {
            // 命令已完成，这里可以添加回调或通知机制
            debug!("AHCI command slot {} completed", slot);
            
            // 清除命令槽的中断标志
            port.interrupt_status.write(1 << slot);
        }
    }
}
```


### 引入UPIntrFreeCell(废案，出现更多报错)
修改kernel/src/sync/mod.rs
```rust
mod condvar;
mod mutex;
mod semaphore;
mod up;  // 这里没有 `pub`，但通过 `pub use` 对外暴露内部项

pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::{UPSafeCell, UPIntrFreeCell, UPIntrRefMut}; // 修改此处
```

在kernel/src/sync/up.rs中定义结构体
```rust
pub struct UPIntrFreeCell<T> {
    /// inner data
    inner: RefCell<T>,
}

unsafe impl<T> Sync for UPIntrFreeCell<T> {}

pub struct UPIntrRefMut<'a, T>(Option<RefMut<'a, T>>);

impl<T> UPIntrFreeCell<T> {
    pub unsafe fn new(value: T) -> Self {
        Self {
            inner: RefCell::new(value),
        }
    }

    /// Panic if the data has been borrowed.
    pub fn exclusive_access(&self) -> UPIntrRefMut<'_, T> {
        INTR_MASKING_INFO.get_mut().enter();
        UPIntrRefMut(Some(self.inner.borrow_mut()))
    }

    pub fn exclusive_session<F, V>(&self, f: F) -> V
    where
        F: FnOnce(&mut T) -> V,
    {
        let mut inner = self.exclusive_access();
        f(inner.deref_mut())
    }
}

impl<'a, T> Drop for UPIntrRefMut<'a, T> {
    fn drop(&mut self) {
        self.0 = None;
        INTR_MASKING_INFO.get_mut().exit();
    }
}

impl<'a, T> Deref for UPIntrRefMut<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        self.0.as_ref().unwrap().deref()
    }
}
impl<'a, T> DerefMut for UPIntrRefMut<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0.as_mut().unwrap().deref_mut()
    }
}
```
