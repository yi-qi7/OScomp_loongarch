# 文件系统迁移为ext4

## 更改的文件

靖宇还在更新文件系统内的代码，此处先复制浩文库内的ext4_rs文件到根目录

复制virtio-drivers到根目录

在根目录创建img文件

修改Makefile文件

修改kernel/Cargo.toml文件

os/src/fs/inode.rs -> kernel/src/fs/inode.rs

os/src/drivers/block/mod.rs -> kernel/src/loongarch/driver/mod.rs代码更改

os/src/drivers/block/virtio_blk.rs -> kernel/src/loongarch/driver/ahci.rs代码更改。同时把ahci文件改名为virtio_blk

## 引入UPIntrFreeCell
修改kernel/src/sync/mod.rs
'''rust
mod condvar;
mod mutex;
mod semaphore;
mod up;  // 这里没有 `pub`，但通过 `pub use` 对外暴露内部项

pub use condvar::Condvar;
pub use mutex::{Mutex, MutexBlocking, MutexSpin};
pub use semaphore::Semaphore;
pub use up::{UPSafeCell, UPIntrFreeCell, UPIntrRefMut}; // 修改此处
'''
