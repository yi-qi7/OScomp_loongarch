# rCore的龙芯平台移植

有关系统调用的迁移与实现位于[变更说明](/变更说明)

## to do
从 busybox/libbb/xfuncs_printf.c 中提取的系统调用有

1. `malloc`
2. `realloc` 
3. `free` 
4. `mmap` 
5. `fopen` 
6. `open` 
7. `unlink` 
8. `rename` 
9. `pipe` 
10. `dup2` 
11. `write` 
12. `close` 
13. `lseek`
15. `mkstemp` 
1. `ferror()`  
2. `fflush()`  
3. `putchar()`  
4. `fclose()`  
5. `malloc()`  
6. `setenv()`  
7. `unsetenv()`  
8. `setgid()`  
9. `setuid()`  
10. `setegid()`  
11. `seteuid()`  
12. `chdir()`  
13. `fchdir()`  
14. `chroot()`  
15. `opendir()`  
16. `socket()`  
17. `bind()`  
18. `listen()`  
19. `sendto()`  
20. `stat()`  
21. `fstat()`  
22. `ioctl()`  
23. `ttyname_r()`  
24. `fork()`  
25. `waitpid()`  
26. `settimeofday()`  
27. `open()`  
28. `read()`  
29. `write()`  
30. `close()`





