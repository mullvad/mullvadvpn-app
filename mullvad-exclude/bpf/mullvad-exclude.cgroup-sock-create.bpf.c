// SPDX-License-Identifier: GPL-2.0-or-later

#include <sys/socket.h>
#include <linux/bpf.h>
#include <bpf/bpf_helpers.h>
#include "fwmark.h"

char LICENSE[] SEC("license") = "GPL";

SEC("cgroup/sock_create")
int sock(struct bpf_sock *ctx)
{
    if (ctx->mark == 0) {
        ctx->mark = FWMARK;
    }

    return 1;
}
