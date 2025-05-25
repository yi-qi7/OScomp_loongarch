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