# Building the Mullvad app for Linksys AC 1200

## Easy

```
$ bash build-armv7.sh
```

## Hardware info

The following is a dump from `uname -a`, `/proc/cpuinfo` and `ldd --version`

```bash
  _______                     ________        __
 |       |.-----.-----.-----.|  |  |  |.----.|  |_
 |   -   ||  _  |  -__|     ||  |  |  ||   _||   _|
 |_______||   __|_____|__|__||________||__|  |____|
          |__| W I R E L E S S   F R E E D O M
 -----------------------------------------------------
 OpenWrt 24.10.2, r28739-d9340319c6
 -----------------------------------------------------
root@OpenWrt:~# uname -a
Linux OpenWrt 6.6.93 #0 SMP Mon Jun 23 20:40:36 2025 armv7l GNU/Linux
root@OpenWrt:~# cat /proc/cpuinfo
processor       : 0
model name      : ARMv7 Processor rev 1 (v7l)
BogoMIPS        : 1332.00
Features        : half thumb fastmult vfp edsp neon vfpv3 tls vfpd32
CPU implementer : 0x41
CPU architecture: 7
CPU variant     : 0x4
CPU part        : 0xc09
CPU revision    : 1

processor       : 1
model name      : ARMv7 Processor rev 1 (v7l)
BogoMIPS        : 1332.00
Features        : half thumb fastmult vfp edsp neon vfpv3 tls vfpd32
CPU implementer : 0x41
CPU architecture: 7
CPU variant     : 0x4
CPU part        : 0xc09
CPU revision    : 1

Hardware        : Marvell Armada 380/385 (Device Tree)
Revision        : 0000
Serial          : 0000000000000000
root@OpenWrt:~# ldd --version
musl libc (armhf)
Version 1.2.5
Dynamic Program Loader
Usage: ldd [options] [--] pathname
```

OpenWRT @ Linksys AC 1200 is running an ARMv7 CPU with MUSL libc.


## Manual build

### Rust Toolchain
Install the `armv7-unknown-linux-musleabihf` target:

```bash
$ rustup target add armv7-unknown-linux-musleabihf
```

TODO
