# rCore的龙芯平台移植

有关系统调用的迁移与实现位于[变更说明](/变更说明)

## to do
从 busybox/libbb/xfuncs_printf.c 中提取的系统调用有

### 内存管理相关系统调用
1. **`xmalloc(size_t size)`**
   - 该函数分配指定大小的内存，并在内存分配失败时终止程序并输出错误信息。
   
2. **`xrealloc(void *ptr, size_t size)`**
   - 用于重新分配内存（改变大小），如果内存重新分配失败，程序会终止并输出错误信息。

3. **`xzalloc(size_t size)`**
   - 分配内存并将其初始化为零。如果内存分配失败，程序会终止并输出错误信息。

4. **`xstrdup(const char *s)`**
   - 复制字符串，并在内存分配失败时终止程序并输出错误信息。

5. **`xstrndup(const char *s, int n)`**
   - 复制字符串的前n个字符，并在内存分配失败时终止程序并输出错误信息。

6. **`xmemdup(const void *s, int n)`**
   - 复制指定大小的内存块，并在内存分配失败时终止程序并输出错误信息。

### 文件操作相关系统调用
1. **`xfopen(const char *path, const char *mode)`**
   - 打开文件并返回文件指针，如果文件打开失败，程序会终止并输出错误信息。

2. **`xopen3(const char *pathname, int flags, int mode)`**
   - 打开文件并返回文件描述符，如果文件打开失败，程序会终止并输出错误信息。

3. **`xopen(const char *pathname, int flags)`**
   - 打开文件并返回文件描述符，默认为`0666`的权限。如果文件打开失败，程序会终止并输出错误信息。

4. **`open3_or_warn(const char *pathname, int flags, int mode)`**
   - 尝试打开文件并返回文件描述符。如果文件打开失败，会输出警告信息，但不会终止程序。

5. **`open_or_warn(const char *pathname, int flags)`**
   - 尝试打开文件并返回文件描述符，默认权限为`0666`。如果文件打开失败，会输出警告信息。

6. **`xunlink(const char *pathname)`**
   - 删除文件。如果删除失败，程序会终止并输出错误信息。

7. **`xrename(const char *oldpath, const char *newpath)`**
   - 重命名文件。如果重命名失败，程序会终止并输出错误信息。

8. **`rename_or_warn(const char *oldpath, const char *newpath)`**
   - 尝试重命名文件。如果重命名失败，会输出警告信息，但不会终止程序。

9. **`xpipe(int filedes[2])`**
   - 创建管道。如果创建失败，程序会终止并输出错误信息。

10. **`xdup2(int from, int to)`**
    - 将文件描述符`from`复制到`to`。如果复制失败，程序会终止并输出错误信息。

11. **`xmove_fd(int from, int to)`**
    - 重命名已打开的文件描述符`from`为`to`，并关闭`from`。如果操作失败，程序会终止并输出错误信息。

12. **`xwrite(int fd, const void *buf, size_t count)`**
    - 将数据写入文件。如果写入失败，程序会终止并输出错误信息。

13. **`xwrite_str(int fd, const char *str)`**
    - 将字符串写入文件。如果写入失败，程序会终止并输出错误信息。

14. **`xclose(int fd)`**
    - 关闭文件描述符。如果关闭失败，程序会终止并输出错误信息。

15. **`xlseek(int fd, off_t offset, int whence)`**
    - 将文件描述符`fd`的偏移量设置为`offset`。如果操作失败，程序会终止并输出错误信息。

16. **`xmkstemp(char *template)`**
    - 创建一个临时文件并返回文件描述符。如果创建失败，程序会终止并输出错误信息。

### 内存映射相关系统调用
1. **`mmap_read(int fd, size_t size)`**
   - 映射文件`fd`为内存以进行读取。

2. **`mmap_anon(size_t size)`**
   - 为匿名映射分配内存。

3. **`xmmap_anon(size_t size)`**
   - 为匿名映射分配内存，并在失败时终止程序并输出错误信息。




