/* SPDX-License-Identifier: GPL-2.0
 *
 * Copyright (C) 2018-2019 WireGuard LLC. All Rights Reserved.
 */

#ifndef WIREGUARD_H
#define WIREGUARD_H

#include <sys/types.h>
#include <stdint.h>
#include <stdbool.h>

typedef struct { const char *p; size_t n; } gostring_t;
typedef void(*logger_fn_t)(int level, const char *msg);
extern void wgEnableRoaming(bool enabled);
extern void wgSetLogger(logger_fn_t logger_fn);
extern int wgTurnOn(gostring_t settings, int32_t tun_fd);
extern void wgTurnOff(int handle);
extern int64_t wgSetConfig(int handle, gostring_t settings);
extern char *wgGetConfig(int handle);
extern void wgBumpSockets(int handle);
extern const char *wgVersion();

#endif
