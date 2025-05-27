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

### kernel/Cargo.toml
不使用virtio-blk-device设备而是使用STATA硬盘模拟，并添加了Ahci协议。
```rust
isomorphic_drivers = { path = "../isomorphic_drivers" } #使用
#virtio-drivers = { path = "../virtio-drivers" }  不使用virtio-drivers
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
