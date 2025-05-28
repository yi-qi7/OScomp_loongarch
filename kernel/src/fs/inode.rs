//! `Arc<Inode>` -> `OSInodeInner`: In order to open files concurrently
//! we need to wrap `Inode` into `Arc`,but `Mutex` in `Inode` prevents
//! file systems from being accessed simultaneously
//!
//! `UPSafeCell<OSInodeInner>` -> `OSInode`: for static `ROOT_INODE`,we
//! need to wrap `OSInodeInner` into `UPSafeCell`
use alloc::{sync::Arc, vec::Vec};

use bitflags::*;

// use easy_fs::{EasyFileSystem, Inode}; //ext4
use ext4_rs::Ext4; //ext4
use alloc::string::*; //ext4
use log::info; //ext4
use core::fmt::Result; //ext4
use crate::fs::inode; //ext4
use alloc::vec; //ext4

use lazy_static::*;

use super::File;
use crate::{loongarch::BLOCK_DEVICE, mm::UserBuffer, println, sync::UPSafeCell};


//ext4
lazy_static! {
    pub static ref EXT4: Ext4 = {
        Ext4::open(Arc::clone(&BLOCK_DEVICE))
    };
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        Arc::new(Inode {
            inode: 2, //Root inode is 2 in ext4
        }) 
    };
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Inode {
    inode: u32,
}

impl Inode {
    fn read_at(&self, offset: usize, buf: &mut [u8]) -> usize {
        let result = EXT4.read_at(self.inode, offset, buf);
        match result {
            Ok(size) => size,
            Err(err) => panic!("ext4 read_at error in os/src/fs/inode: {:?}", err),
        }
    }

    fn write_at(&self, offset: usize, buf: &[u8]) -> usize {
        let result = EXT4.write_at(self.inode, offset, &buf);
        match result {
            Ok(size) => size,
            Err(err) => panic!("ext4 write_at error in os/src/fs/inode: {:?}", err),
        }
    }

    fn get_dirstr(&self) -> String {
        let mut inode = self.inode;
        let mut path_parts: Vec<String> = Vec::new();

        loop {
            // 获取当前目录的所有目录项
            let dirents = EXT4.dir_get_entries(inode);
            self.get_dirents();
            // 找到".."，拿到parent_inode
            let mut parent_inode = inode;
            for dirent in &dirents {
                // log::info!("print dir {}",dirent.get_name());
                // log::info!(" inode num: {}", dirent.inode);
                if dirent.compare_name("..") {
                    // log::info!(".. inode {}", dirent.inode);
                    parent_inode = dirent.inode;
                    break;
                }
            }
            // 到达根目录，停止
            if inode == parent_inode {
                break;
            }

            // 在父目录查找，找到自己的名字
            let parent_dirents = EXT4.dir_get_entries(parent_inode);
            let mut found = false;
            for dirent in &parent_dirents {
                // log::info!("print dir2 {}",dirent.get_name());
                if dirent.inode == inode && !dirent.compare_name(".") && !dirent.compare_name("..") {
                    path_parts.push(dirent.get_name().to_string());
                    found = true;
                    break;
                }
            }

            if !found {
                // 没找到，可能目录损坏，返回"/"
                return "/".to_string();
            }

            inode = parent_inode;
        }

        if path_parts.is_empty() {
            "/".to_string()
        } else {
            // 逆序输出
            let mut result = String::new();
            for name in path_parts.iter().rev() {
                result.push('/');
                result.push_str(name);
            }
            result
        }
    }


    fn get_dirents(&self) {
        let entries = EXT4.dir_get_entries(self.inode);
        for entry in entries {
            println!("{:?}",entry.get_name());
        }
    }

    //ext4
    pub fn ls(&self) -> Vec<String> {
        // 具体实现在sasy-fs/src/vfs.rs中，但迁移有问题
        vec![] // 返回空列表
    }

}

pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>, //将所有UPIntrFreeCell改为PSafeCell
}

pub struct OSInodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

impl OSInode {
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPSafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.exclusive_access();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

// write_offset: start_block_id: 262144, write_len: 4096, offset: 134217728, data_addr: 8124e000, fit_len: 4096, temp buf len 4096

// write_offset: start_block_id: 262144, write_len: 4096, offset: 134217728, data_addr: 8124d000, fit_len: 4096, temp buf len 4096

pub fn list_apps() {
    println!("/**** FileSystemTest ****");
    let test = vec![6u8; 4096];
    BLOCK_DEVICE.write_offset(134217728,&test);
    let res = EXT4.ext4_dir_mk("/test/test1/test2");
    res.unwrap();
    ROOT_INODE.get_dirents();    
    println!("**************/");
    println!("open file test");
    let result = EXT4.generic_open("/test_open", &mut 2, true, 0, &mut 0);
    let result2 = EXT4.generic_open("/test_open2", &mut 2, true, 0, &mut 0);
    let result3 = EXT4.generic_open("/test_open3", &mut 2, true, 0, &mut 0);
    match result {
        Ok(inode) => {
            println!("open file success, inode: {}", inode);
        },
        Err(err) => {
            println!("open file error: {:?}", err);
        }
    }
    match result2 {
        Ok(inode) => {
            println!("open file success, inode: {}", inode);
        },
        Err(err) => {
            println!("open file error: {:?}", err);
        }
    }    
    match result3 {
        Ok(inode) => {
            println!("open file success, inode: {}", inode);
        },
        Err(err) => {
            println!("open file error: {:?}", err);
        }
    }
    let res_inode = mkdir("/test_path/test_path2/../test_path2/test_path3");
    println!("res_inode of mkdir = {:?}", res_inode);
    let result = EXT4.generic_open("/test_path/test_path2/test_path3/testfile2332", &mut 2, true, 0, &mut 0);
    println!("make /test_path/test_path2/test_path3/testfile2332 success, inode: {}", result.unwrap());
    ROOT_INODE.get_dirents(); 
    assert!(res_inode > 0);
    let dir_inode = Inode { inode: res_inode as u32 };
    log::info!("dir_inode = {:?}", dir_inode);
    dir_inode.get_dirents();
    println!("get path from inode = {:?}", dir_inode.get_dirstr());
}

bitflags! {
    pub struct OpenFlags: u32 {
        const RDONLY = 0;
        const WRONLY = 1 << 0;
        const RDWR = 1 << 1;
        const CREATE = 1 << 9;
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
}

//Ciallo~
pub fn open_file(path: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    println!("open_file");
    let (readable, writable) = flags.read_write();
    let mut open_args = "r+";
    if flags.contains(OpenFlags::TRUNC) || flags.contains(OpenFlags::CREATE) {
        open_args = "w+";
        // println!("path = {:?}",path);
        // let result = EXT4.generic_open(path, &mut 2, true, 0, &mut 0);
        // return Some(Arc::new(OSInode::new(readable, writable, Arc::new(Inode { inode: result.unwrap() }))));
    }
    println!("path:{}",path);
    let inode = EXT4.ext4_file_open(path, open_args);
    println!("end_open_file");
    match inode {
        Ok(inode_num) => {
            Some(Arc::new(OSInode::new(readable, writable, Arc::new(Inode { inode: inode_num }))))
        },
        Err(_) => {
            None
        }
    }
}

pub fn mkdir(path: &str) -> isize {
    let inode = EXT4.ext4_dir_mk(path);
    match inode {
        Ok(inode_num) => {
            inode_num as isize
        },
        Err(_) => {
            -1
        }
    }
}

pub fn rmdir() {
    unimplemented!()
}


// A wrapper around a filesystem inode
// to implement File trait atop
//ext4
/* pub struct OSInode {
    readable: bool,
    writable: bool,
    inner: UPSafeCell<OSInodeInner>,
}
/// The OS inode inner in 'UPSafeCell'
pub struct OSInodeInner {
    offset: usize,
    inode: Arc<Inode>,
}

impl OSInode {
    /// Construct an OS inode from a inode
    pub fn new(readable: bool, writable: bool, inode: Arc<Inode>) -> Self {
        Self {
            readable,
            writable,
            inner: unsafe { UPSafeCell::new(OSInodeInner { offset: 0, inode }) },
        }
    }
    /// Read all data inside a inode into vector
    pub fn read_all(&self) -> Vec<u8> {
        let mut inner = self.inner.exclusive_access();
        let mut buffer = [0u8; 512];
        let mut v: Vec<u8> = Vec::new();
        loop {
            let len = inner.inode.read_at(inner.offset, &mut buffer);
            if len == 0 {
                break;
            }
            inner.offset += len;
            v.extend_from_slice(&buffer[..len]);
        }
        v
    }
}

lazy_static! {
    pub static ref ROOT_INODE: Arc<Inode> = {
        let efs = EasyFileSystem::open(BLOCK_DEVICE.clone());
        Arc::new(EasyFileSystem::root_inode(&efs))
    };
}
/// List all files in the filesystems
pub fn list_apps() {
    println!("/**** APPS ****");
    for app in ROOT_INODE.ls() {
        println!("{}", app);
    }
    println!("**************/");
}

bitflags! {
    ///Open file flags
    pub struct OpenFlags: u32 {
        ///Read only
        const RDONLY = 0;
        ///Write only
        const WRONLY = 1 << 0;
        ///Read & Write
        const RDWR = 1 << 1;
        ///Allow create
        const CREATE = 1 << 9;
        ///Clear file and return an empty one
        const TRUNC = 1 << 10;
    }
}

impl OpenFlags {
    /// Do not check validity for simplicity
    /// Return (readable, writable)
    pub fn read_write(&self) -> (bool, bool) {
        if self.is_empty() {
            (true, false)
        } else if self.contains(Self::WRONLY) {
            (false, true)
        } else {
            (true, true)
        }
    }
}
///Open file with flags
pub fn open_file(name: &str, flags: OpenFlags) -> Option<Arc<OSInode>> {
    let (readable, writable) = flags.read_write();
    if flags.contains(OpenFlags::CREATE) {
        if let Some(inode) = ROOT_INODE.find(name) {
            // clear size
            inode.clear();
            Some(Arc::new(OSInode::new(readable, writable, inode)))
        } else {
            // create file
            ROOT_INODE
                .create(name)
                .map(|inode| Arc::new(OSInode::new(readable, writable, inode)))
        }
    } else {
        ROOT_INODE.find(name).map(|inode| {
            if flags.contains(OpenFlags::TRUNC) {
                inode.clear();
            }
            Arc::new(OSInode::new(readable, writable, inode))
        })
    }
}

impl File for OSInode {
    fn readable(&self) -> bool {
        self.readable
    }
    fn writable(&self) -> bool {
        self.writable
    }
    fn read(&self, mut buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_read_size = 0usize;
        for slice in buf.buffers.iter_mut() {
            let read_size = inner.inode.read_at(inner.offset, *slice);
            if read_size == 0 {
                break;
            }
            inner.offset += read_size;
            total_read_size += read_size;
        }
        total_read_size
    }
    fn write(&self, buf: UserBuffer) -> usize {
        let mut inner = self.inner.exclusive_access();
        let mut total_write_size = 0usize;
        for slice in buf.buffers.iter() {
            let write_size = inner.inode.write_at(inner.offset, *slice);
            assert_eq!(write_size, slice.len());
            inner.offset += write_size;
            total_write_size += write_size;
        }
        total_write_size
    }
} */
