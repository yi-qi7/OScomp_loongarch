# rCore的龙芯平台移植
[rCore仓库](https://github.com/YusanXY/OScomp/tree/dev_yiqi7)

有关系统调用的迁移与实现位于[变更说明](/变更说明)

## to do
[需要实现的系统调用](https://github.com/YusanXY/OScomp/blob/dev_yhw/syscalls_list.md#%E7%B3%BB%E7%BB%9F%E8%B0%83%E7%94%A8-in-busybox)
- [ ] prctl
- [ ] getuid
- [ ] fstat
- [ ] fcntl
- [x] rename
- [x] uname
- [ ] mmap
- [ ] geteuid
- [ ] ulink
- [ ] dup2
- [x] dup3
- [ ] lseek
- [ ] setgid
- [ ] setuid
- [ ] setegid
- [ ] seteuid
- [ ] chroot
- [x] socket 测试通过
- [ ] bind
- [ ] listen  这个v3有实现，考虑将v3的实现迁移过来
- [ ] sendto
- [ ] stat
- [ ] ioctl
- [ ] vfork
- [ ] settimeofday






