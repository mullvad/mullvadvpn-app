bitflags::bitflags! {
    /// Routing message flags
    /// See https://www.manpagez.com/man/4/route/.
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub struct RouteFlag: i32 {
        /// route usable
        const RTF_UP = libc::RTF_UP;
        /// destination is a gateway
        const RTF_GATEWAY = libc::RTF_GATEWAY;
        /// host entry (net otherwise)
        const RTF_HOST = libc::RTF_HOST;
        /// host or net unreachable
        const RTF_REJECT = libc::RTF_REJECT;
        /// created dynamically (by redirect)
        const RTF_DYNAMIC = libc::RTF_DYNAMIC;
        /// modified dynamically (by redirect)
        const RTF_MODIFIED = libc::RTF_MODIFIED;
        /// message confirmed
        const RTF_DONE = libc::RTF_DONE;
        /// delete cloned route
        const RTF_DELCLONE = libc::RTF_DELCLONE;
        /// generate new routes on use
        const RTF_CLONING = libc::RTF_CLONING;
        /// external daemon resolves name
        const RTF_XRESOLVE = libc::RTF_XRESOLVE;
        /// used by apps to add/del L2 entries
        /// the newer constant is called RTF_LLDATA but absent in libc, has the same value.
        const RTF_LLINFO = libc::RTF_LLINFO;
        /// manually added
        const RTF_STATIC = libc::RTF_STATIC;
        /// just discard pkts (during updates)
        const RTF_BLACKHOLE = libc::RTF_BLACKHOLE;
        /// not eligible for RTF_IFREF
        const RTF_NOIFREF = libc::RTF_NOIFREF;
        /// protocol specific routing flag
        const RTF_PROTO2 = libc::RTF_PROTO2;
        /// protocol specific routing flag
        const RTF_PROTO1 = libc::RTF_PROTO1;
        /// protocol requires cloning
        const RTF_PRCLONING = libc::RTF_PRCLONING;
        /// route generated through cloning
        const RTF_WASCLONED = libc::RTF_WASCLONED;
        /// protocol specific routing flag
        const RTF_PROTO3 = libc::RTF_PROTO3;
        /// future use
        const RTF_PINNED = libc::RTF_PINNED;
        /// route represents a local address
        const RTF_LOCAL = libc::RTF_LOCAL;
        /// route represents a bcast address
        const RTF_BROADCAST = libc::RTF_BROADCAST;
        /// route represents a mcast address
        const RTF_MULTICAST = libc::RTF_MULTICAST;
        /// has valid interface scope
        const RTF_IFSCOPE = libc::RTF_IFSCOPE;
        /// defunct; no longer modifiable
        const RTF_CONDEMNED = libc::RTF_CONDEMNED;
        /// route holds a ref to interface
        const RTF_IFREF = libc::RTF_IFREF;
        /// proxying, no interface scope
        const RTF_PROXY = libc::RTF_PROXY;
        /// host is a router
        const RTF_ROUTER = libc::RTF_ROUTER;
        /// Route entry is being freed
        const RTF_DEAD = libc::RTF_DEAD;
        /// route to destination of the global internet
        const RTF_GLOBAL = libc::RTF_GLOBAL;
    }
}
