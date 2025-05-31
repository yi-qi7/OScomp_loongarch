# socket

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
- domain：即协议域，又称为协议族（family）。常用的协议族有，AF_INET、AF_INET6、AF_LOCAL（或称AF_UNIX，Unix域socket）、AF_ROUTE等等。协议族决定了socket的地址类型，在通信中必须采用对应的地址，如AF_INET决定了要用ipv4地址（32位的）与端口号（16位的）的组合、AF_UNIX决定了要用一个绝对路径名作为地址。
- type：指定socket类型。常用的socket类型有，SOCK_STREAM、SOCK_DGRAM、SOCK_RAW、SOCK_PACKET、SOCK_SEQPACKET等等（socket的类型有哪些？）。
- protocol：顾名思义，就是指定协议。常用的协议有，IPPROTO_TCP、IPPTOTO_UDP、IPPROTO_SCTP、IPPROTO_TIPC等，它们分别对应TCP传输协议、UDP传输协议、STCP传输协议、TIPC传输协议


