# socket

**所有更改均有zsq_socket注释**

## 介绍

网络中进程之间的通信。用三元组（ip地址，协议，端口）来标识网络的进程。由于“一切皆文件”的思想，socket即是一种特殊的文件，一些socket函数就是对其进行的操作（读/写IO、打开、关闭）

socket函数对应于普通文件的打开操作。普通文件的打开操作返回一个文件描述字，而socket()用于创建一个socket描述符（socket descriptor），它唯一标识一个socket。这个socket描述字跟文件描述字一样，后续的操作都有用到它，把它作为参数，通过它来进行一些读写操作。

在busybox中的调用为

```rust
int FAST_FUNC xsocket(int domain, int type, int protocol)
{
	int r = socket(domain, type, protocol);

	if (r < 0) {
		/* Hijack vaguely related config option */
#if ENABLE_VERBOSE_RESOLUTION_ERRORS
		const char *s = "INET";
# ifdef AF_PACKET
		if (domain == AF_PACKET) s = "PACKET";
# endif
# ifdef AF_NETLINK
		if (domain == AF_NETLINK) s = "NETLINK";
# endif
IF_FEATURE_IPV6(if (domain == AF_INET6) s = "INET6";)
		bb_perror_msg_and_die("socket(AF_%s,%d,%d)", s, type, protocol);
#else
		bb_simple_perror_msg_and_die("socket");
#endif
	}

	return r;
}
```
socket函数的三个参数分别为
```rust
pub fn sys_socket(domain: u32, typ: u32, protocol: u32) -> isize{}
```
- domain：即协议域，又称为协议族（family）。常用的协议族有，AF_INET、AF_INET6、AF_LOCAL（或称AF_UNIX，Unix域socket）、AF_ROUTE等等。协议族决定了socket的地址类型，在通信中必须采用对应的地址，如AF_INET决定了要用ipv4地址（32位的）与端口号（16位的）的组合、AF_UNIX决定了要用一个绝对路径名作为地址。在这里，我们的协议族选择使用AF_INET
- type：指定socket类型。常用的socket类型有，SOCK_STREAM、SOCK_DGRAM、SOCK_RAW、SOCK_PACKET、SOCK_SEQPACKET等等。我们选择使用的有SOCK_STREAM和SOCK_DGRAM，分别对应TCP和UDP协议
- protocol：顾名思义，就是指定协议。常用的协议有，IPPROTO_TCP、IPPTOTO_UDP、IPPROTO_SCTP、IPPROTO_TIPC等，它们分别对应TCP传输协议、UDP传输协议、STCP传输协议、TIPC传输协议。我们选择使用的有IPPROTO_TCP和IPPROTO_UDP
定义
```rust
// 地址族
pub const AF_INET: u32 = 2;
// Socket类型
pub const SOCK_STREAM: u32 = 1; // TCP
pub const SOCK_DGRAM: u32 = 2;  // UDP
// 协议
pub const IPPROTO_TCP: u32 = 6;
pub const IPPROTO_UDP: u32 = 17;
```
当我们调用socket创建一个socket时，成功会返回文件描述符，失败返回-1

## 修改文件
- os/src/syscall/net.rs -> sys_socket()
- os/src/net/port_table.rs -> 更新drop以应对无效索引的情况
- user/src/bin/net_src/bin/net_socket_test.rs 用于测试

## 实现
os/src/syscall/net.rs
```rust
pub fn sys_socket(domain: u32, typ: u32, protocol: u32) -> isize {
    // 仅支持 IPv4
    if domain != AF_INET {
        return -1;
    }
    
    // 仅支持 TCP 和 UDP
     match (typ, protocol) {
        (SOCK_STREAM, IPPROTO_TCP) => create_tcp_socket(),
        (SOCK_DGRAM, IPPROTO_UDP) => create_udp_socket(),
        _ => {
            -1
        }
    }
}
```
创建套接字，并根据socket类型和协议选择创建TCP协议还是UDP协议

若是TCP协议
```rust
fn create_tcp_socket() -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();

    // 分配文件描述符
    let fd = inner.alloc_fd();

    // 创建新的 TCP Socket 实例
    let socket = Socket {
        raddr: IPv4::new(0, 0, 0, 0),    // 远程地址（初始化为0）
        lport: 0,                        // 本地端口（将在 bind 或 connect 时分配）
        rport: 0,                        // 远程端口（初始化为0）
        buffers: VecDeque::new(),        // 数据缓冲区
        seq: 1,                          // 初始序列号（从1开始符合TCP规范）
        ack: 0,                          // 初始确认号
    };

    // 添加到 Socket 表
    match add_socket(socket.raddr, socket.lport, socket.rport) {
        Some(socket_index) => {
            // 设置初始序列号和确认号
            set_s_a_by_index(socket_index, socket.seq, socket.ack);
            /*if let Err(e) = set_s_a_by_index(socket_index, socket.seq, socket.ack) {
                // 回滚：移除已添加的 socket
                remove_socket(socket_index);
                return -1;
            }*/

            // 创建 PortFd 并关联到 socket_index
            let port_fd = PortFd::new(socket_index);
            inner.fd_table[fd] = Some(Arc::new(port_fd));

            fd as isize
        }
        None => {
            -1
        }
    }
}
```
TCP 连接需要跨进程 / 内核层共享状态，因此需要通过add_socket将实例添加到全局 Socket 表，便于通过socket_index统一管理。


若是UDP协议
```rust
fn create_udp_socket() -> isize {
    let process = current_process();
    let mut inner = process.inner_exclusive_access();

    // 分配文件描述符
    let fd = inner.alloc_fd();

    // 创建 UDP 节点
    // 初始状态：本地地址和端口暂时为 1024，将在 bind 或 connect 时分配
    let udp_node = match UDP::new(
        IPv4::new(0, 0, 0, 0), // 远程地址（初始化为0）
        1024,                     // 本地端口（将在 bind 时分配）
        0,                     // 远程端口（初始化为0）
    ) {
        udp => udp, // UDP::new 目前没有错误处理，直接使用结果
    };

    // 将 UDP 节点添加到文件描述符表
    inner.fd_table[fd] = Some(Arc::new(udp_node));

    fd as isize
}
```
UDP 不需要全局状态表，直接将实例关联到文件描述符表（fd_table）即可，每个 FD 对应独立的 UDP 节点，减少了数据结构开销，不需要通过socket管理

---
os/src/net/port_table.rs
单独测试socket时索引是无效状态，drop会引起panic，所以更新drop处理无效索引的情况

```rust
/*impl Drop for PortFd {
    fn drop(&mut self) {
        LISTEN_TABLE.exclusive_access()[self.0] = None
    }
}*/
//应对无效索引的情况
impl Drop for PortFd {
    fn drop(&mut self) {
        let mut listen_table = LISTEN_TABLE.exclusive_access();
        
        // 检查索引是否有效
        if self.0 >= listen_table.len() {
            
            return;
        }
        
        // 安全地设置为 None
        listen_table[self.0] = None;
    }
}
```

---
user/src/bin/net_src/bin/net_socket_test.rs

测试代码为

```rust
#![no_std]
#![no_main]

use alloc::string::String;

#[macro_use]
extern crate user_lib;
#[macro_use]
extern crate alloc;

// 定义网络常量
const AF_INET: u32 = 2;
const SOCK_STREAM: u32 = 1;
const SOCK_DGRAM: u32 = 2;
const IPPROTO_TCP: u32 = 6;
const IPPROTO_UDP: u32 = 17;

use user_lib::{socket, println};

#[unsafe(no_mangle)]
fn main() -> i32 {
    println!("begin sys_socket test...");
    
    // 测试创建 TCP 套接字
    let tcp_fd = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    if tcp_fd >= 0 {
        println!("TCP success wonderful!: {}", tcp_fd);
    } else {
        println!("TCP failed on no: {}", tcp_fd);
    }
    
    // 测试创建 UDP 套接字
    let udp_fd = socket(AF_INET, SOCK_DGRAM, IPPROTO_UDP);
    if udp_fd >= 0 {
        println!("UDP success wonderful!: {}", udp_fd);
    } else {
        println!("UDP failed on no: {}", udp_fd);
    }
    
    println!("socket end");

    0
}
```

## 参考

https://zhuanlan.zhihu.com/p/100151937


