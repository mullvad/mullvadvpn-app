// SPDX-License-Identifier: GPL-2.0-or-later

#include <sys/socket.h>
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include "fwmark.h"

char LICENSE[] SEC("license") = "GPL";

// Set FWMARK on all newly created sockets to exclude them from the tunnel.
SEC("cgroup/sock_create")
int mullvad_exclude_sock_create(struct bpf_sock *ctx) {
    ctx->mark = FWMARK;
    return 1;
}

// Forbid applications in the cgroup from setting SO_MARK and breaking split-tunneling.
SEC("cgroup/setsockopt")
int mullvad_exclude_deny_so_mark(struct bpf_sockopt *ctx) {
    return !(ctx->level == SOL_SOCKET && ctx->optname == SO_MARK);
}
