# socket

网络中进程之间的通信。用三元组（ip地址，协议，端口）来标识网络的进程。由于“一切皆文件”的思想，socket即是一种特殊的文件，一些socket函数就是对其进行的操作（读/写IO、打开、关闭）

socket函数对应于普通文件的打开操作。普通文件的打开操作返回一个文件描述字，而socket()用于创建一个socket描述符（socket descriptor），它唯一标识一个socket。这个socket描述字跟文件描述字一样，后续的操作都有用到它，把它作为参数，通过它来进行一些读写操作。

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

