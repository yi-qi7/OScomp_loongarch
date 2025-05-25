# SYS_times 153

具体实现思路见[浩文的分支](https://github.com/YusanXY/OScomp/blob/dev_yhw/README.md)这里仅记录迁移到龙芯上后需要更改的地方


在 `kernel/src/timer.rs` 中，添加结构体`Times`和以下内容
```rust
#[repr(C)] //保证与C语言的内存分布一致
pub struct Times{
    pub utime:isize,
    pub stime:isize,
    pub cutime:isize,
    pub cstime:isize,
    pub u_start_time:isize,
    pub s_start_time:isize,
}
impl Clone for Times {
    fn clone(&self) -> Self {
        Self {
            utime: self.utime,
            stime: self.stime,
            cutime: self.cutime,
            cstime: self.cstime,
            u_start_time: self.u_start_time,
            s_start_time: self.s_start_time,
        }
    }
}
impl Times{
    pub fn new()->Times{
        Times{
            utime:0,
            stime:0,
            cutime:0,
            cstime:0,
            u_start_time:get_time_ms() as isize,
            s_start_time:get_time_ms() as isize,
        }
    }
}
#[repr(C)]
pub struct Timespec {
    pub sec:usize,
    pub usec:usize,
}
```

在`os/src/task/process.rs`中的 `ProcessControlBlockInner`内添加：
```rust
pub struct ProcessControlBlockInner {
    //...
    pub times:Times,
    pub in_user:bool,//true if in user,false if in kernel
}
```
之后，下面的 ProcessControlBlock也要进行相应的更改（pcb中添加相应的项）
```rust
UPIntrFreeCell::new(ProcessControlBlockInner {
                    //...
                    times:Times::new(),     //新增代码
                    in_user:true,           //新增代码
                })
```
同样的，后面的 fork 中创建子进程pcb也需要添加相应项
```rust
// create child process pcb
        let child = Arc::new(Self {
            pid,
            inner: unsafe {
                UPSafeCell::new(ProcessControlBlockInner {
                    //...
                    times:Times::new(),
                    in_user:true,
                })
            },
        });
```

在`os/src/task/process.rs`中添加以下时间更新相关的函数
```rust
pub fn update_utime(&mut self)->isize{
	if self.in_user==true {
		let curr_time=get_time_ms() as isize;
		let utime_increment= curr_time-self.times.u_start_time;
		self.times.utime+=utime_increment;
		self.times.u_start_time = curr_time;
		utime_increment
	}
	else{
		0
	}
}
pub fn get_utime(&mut self)->isize{
	self.update_utime();
	self.times.utime
}
pub fn restart_utime(&mut self){
	if self.in_user==false{
		self.in_user=true;
		self.times.u_start_time=get_time_ms() as isize;
	}
}
pub fn update_stime(&mut self)->isize{
	if self.in_user==false {//in kernel
		let curr_time=get_time_ms() as isize;
		let stime_increment = curr_time-self.times.s_start_time;
		self.times.stime+=stime_increment;
		self.times.s_start_time=curr_time;
		stime_increment
	}
	else{
		0
	}
}
pub fn get_stime(&mut self) ->isize{
	self.update_stime();
	self.times.stime
}
pub fn restart_stime(&mut self){
	if self.in_user==true{
		self.in_user=false;
		self.times.s_start_time=get_time_ms() as isize;
	}
}
pub fn get_cutime(&mut self)->isize{
	if self.children.len()>0 {
		let mut cutime_temp=0;
		for child in &mut self.children {
			cutime_temp+=child.inner_exclusive_access().get_utime();
		}
		self.times.cutime=cutime_temp;
	}
	self.times.cutime
}
pub fn get_cstime(&mut self)->isize{
	if self.children.len()>0 {
		let mut cstime_temp=0;
		for child in &mut self.children {
			cstime_temp+=child.inner_exclusive_access().get_stime();
		}
		self.times.cstime=cstime_temp;
	}
	self.times.cstime
}
```

在`kernel/src/syscall/process.rs`中，实现系统调用`sys_times`
```rust
use crate::timer::Times;
pub fn sys_times(tms_buf: *mut Times)->isize{
    let token = current_user_token();
    let tms_buf_ptr = translated_refmut(token, tms_buf);
    let curr_process=current_process();
    
    let mut process_inner=curr_process.inner_exclusive_access();
    (*tms_buf_ptr).utime=process_inner.get_utime() as isize;//the size of "isize" is 64bits
    (*tms_buf_ptr).stime=process_inner.get_stime() as isize;
    (*tms_buf_ptr).cutime= process_inner.get_cutime() as isize;
    (*tms_buf_ptr).cstime= process_inner.get_cstime() as isize;

    let curr_time=get_time_ms() as isize;
    if curr_time>=0 {
        curr_time
    }else{
        -1
    }
}
```

在`kernel/src/task/process.rs`中添加
```rust
//...
use crate::timer::Times;
use crate::get_time_ms;
//...
```

在`kernel/src/syscall/mod.rs`中添加
```rust
const SYSCALL_TIMES:usize =153;
//...
use crate::timer::Times;
//...
SYSCALL_TIMES => sys_times(args[0] as *mut Times),
```

在`user/src/syscall.rs`中添加系统调用
```rust
//...
const SYSCALL_TIMES:usize =153;
//...
use crate::times::*;
pub fn sys_times(tms_buf:*const Times)->isize{
    syscall(SYSCALL_TIMES,tms_buf as usize,0,0) //此处有更改
}
```

在`user/src/lib.rs`中添加
```rust
//...
pub use times::*;//YHW
//...
```

在`user/src/times.rs`中添加(times.rx需新建)
```rust
use super::*;
#[repr(C)] //To comfirm the memory structure is the same as that in C
pub struct Times{
    pub tms_utime:isize,
    pub tms_stime:isize,
    pub tms_cutime:isize,
    pub tms_cstime:isize,
}
pub fn times(tms_ptr: *const Times)->isize{
    sys_times(tms_ptr)
}
#[repr(C)]
pub struct Timespec {
    pub sec:usize,
    pub usec:usize,
}
impl Timespec{
    pub fn new()->Timespec{
        Timespec{
            sec:0,
            usec:0,
        }
    }
}
```



