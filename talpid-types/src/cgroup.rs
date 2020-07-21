use std::{ffi::OsStr, fs, os::unix::ffi::OsStrExt, path::PathBuf};

pub const SPLIT_TUNNEL_CGROUP_NAME: &str = "mullvad-exclusions";

/// Find the path of the cgroup v1 net_cls controller mount if it exists
pub fn find_net_cls_mount() -> std::io::Result<Option<PathBuf>> {
    let mounts = fs::read("/proc/mounts")?;
    Ok(find_net_cls_mount_inner(&mounts))
}

fn find_net_cls_mount_inner(mounts: &[u8]) -> Option<PathBuf> {
    mounts
        .split(|byte| *byte == b'\n')
        .find_map(parse_mount_line)
}

fn parse_mount_line(line: &[u8]) -> Option<PathBuf> {
    // Each line contains multiple values seperated by space.
    // `cgroup /sys/fs/cgroup/net_cls,net_prio cgroup
    // rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0`  Value meanings:
    // 1. device type
    // 2. mount path
    // 3. filesystem type
    // 4. mount options
    // 5./6. legacy dummy values
    let mut parts = line.split(|byte| *byte == b' ');
    let _device_type = parts.next()?;
    let mount_path = parts.next()?;
    let filesystem_type = parts.next()?;
    let mount_options = parts.next()?;
    // The expected device type and fs type is "cgroup";
    if filesystem_type != b"cgroup" {
        return None;
    }

    if !mount_options
        .split(|byte| *byte == b',')
        .any(|key| key == b"net_cls")
    {
        return None;
    }

    Some(PathBuf::from(OsStr::from_bytes(mount_path)))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_find_net_cls_path() {
        let input = br#"sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
udev /dev devtmpfs rw,nosuid,noexec,relatime,size=989436k,nr_inodes=247359,mode=755 0 0
devpts /dev/pts devpts rw,nosuid,noexec,relatime,gid=5,mode=620,ptmxmode=000 0 0
tmpfs /run tmpfs rw,nosuid,nodev,noexec,relatime,size=203520k,mode=755 0 0
/dev/vda5 / ext4 rw,relatime,errors=remount-ro 0 0
securityfs /sys/kernel/security securityfs rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /dev/shm tmpfs rw,nosuid,nodev 0 0
tmpfs /run/lock tmpfs rw,nosuid,nodev,noexec,relatime,size=5120k 0 0
tmpfs /sys/fs/cgroup tmpfs ro,nosuid,nodev,noexec,mode=755 0 0
cgroup2 /sys/fs/cgroup/unified cgroup2 rw,nosuid,nodev,noexec,relatime,nsdelegate 0 0
cgroup /sys/fs/cgroup/systemd cgroup rw,nosuid,nodev,noexec,relatime,xattr,name=systemd 0 0
pstore /sys/fs/pstore pstore rw,nosuid,nodev,noexec,relatime 0 0
none /sys/fs/bpf bpf rw,nosuid,nodev,noexec,relatime,mode=700 0 0
cgroup /sys/fs/cgroup/blkio cgroup rw,nosuid,nodev,noexec,relatime,blkio 0 0
cgroup /sys/fs/cgroup/memory cgroup rw,nosuid,nodev,noexec,relatime,memory 0 0
cgroup /sys/fs/cgroup/hugetlb cgroup rw,nosuid,nodev,noexec,relatime,hugetlb 0 0
cgroup /sys/fs/cgroup/cpuset cgroup rw,nosuid,nodev,noexec,relatime,cpuset 0 0
cgroup /sys/fs/cgroup/cpu,cpuacct cgroup rw,nosuid,nodev,noexec,relatime,cpu,cpuacct 0 0
cgroup /sys/fs/cgroup/pids cgroup rw,nosuid,nodev,noexec,relatime,pids 0 0
cgroup /sys/fs/cgroup/rdma cgroup rw,nosuid,nodev,noexec,relatime,rdma 0 0
cgroup /sys/fs/cgroup/net_cls,net_prio cgroup rw,nosuid,nodev,noexec,relatime,net_cls,net_prio 0 0
cgroup /sys/fs/cgroup/devices cgroup rw,nosuid,nodev,noexec,relatime,devices 0 0
cgroup /sys/fs/cgroup/freezer cgroup rw,nosuid,nodev,noexec,relatime,freezer 0 0
cgroup /sys/fs/cgroup/perf_event cgroup rw,nosuid,nodev,noexec,relatime,perf_event 0 0
systemd-1 /proc/sys/fs/binfmt_misc autofs rw,relatime,fd=28,pgrp=1,timeout=0,minproto=5,maxproto=5,direct,pipe_ino=14329 0 0
hugetlbfs /dev/hugepages hugetlbfs rw,relatime,pagesize=2M 0 0
mqueue /dev/mqueue mqueue rw,nosuid,nodev,noexec,relatime 0 0
debugfs /sys/kernel/debug debugfs rw,nosuid,nodev,noexec,relatime 0 0
tracefs /sys/kernel/tracing tracefs rw,nosuid,nodev,noexec,relatime 0 0
fusectl /sys/fs/fuse/connections fusectl rw,nosuid,nodev,noexec,relatime 0 0
configfs /sys/kernel/config configfs rw,nosuid,nodev,noexec,relatime 0 0
/dev/loop1 /snap/gnome-3-34-1804/24 squashfs ro,nodev,relatime 0 0
/dev/loop2 /snap/core18/1880 squashfs ro,nodev,relatime 0 0
/dev/loop3 /snap/gtk-common-themes/1506 squashfs ro,nodev,relatime 0 0
/dev/loop0 /snap/core18/1754 squashfs ro,nodev,relatime 0 0
/dev/loop5 /snap/snap-store/467 squashfs ro,nodev,relatime 0 0
/dev/loop6 /snap/gnome-3-34-1804/36 squashfs ro,nodev,relatime 0 0
/dev/loop7 /snap/snap-store/454 squashfs ro,nodev,relatime 0 0
/dev/loop8 /snap/snapd/8140 squashfs ro,nodev,relatime 0 0
/dev/vda1 /boot/efi vfat rw,relatime,fmask=0077,dmask=0077,codepage=437,iocharset=iso8859-1,shortname=mixed,errors=remount-ro 0 0
tmpfs /run/user/125 tmpfs rw,nosuid,nodev,relatime,size=203516k,mode=700,uid=125,gid=130 0 0
gvfsd-fuse /run/user/125/gvfs fuse.gvfsd-fuse rw,nosuid,nodev,relatime,user_id=125,group_id=130 0 0
/dev/loop9 /snap/snapd/8542 squashfs ro,nodev,relatime 0 0
tmpfs /run/user/1000 tmpfs rw,nosuid,nodev,relatime,size=203516k,mode=700,uid=1000,gid=1000 0 0
gvfsd-fuse /run/user/1000/gvfs fuse.gvfsd-fuse rw,nosuid,nodev,relatime,user_id=1000,group_id=1000 0 0
some-garbage-line
"#;

        assert_eq!(
            find_net_cls_mount_inner(input),
            Some(PathBuf::from("/sys/fs/cgroup/net_cls,net_prio"))
        )
    }

    #[test]
    fn test_fail_to_find_net_cls_path() {
        let input = br#"sysfs /sys sysfs rw,nosuid,nodev,noexec,relatime 0 0
proc /proc proc rw,nosuid,nodev,noexec,relatime 0 0
udev /dev devtmpfs rw,nosuid,noexec,relatime,size=989436k,nr_inodes=247359,mode=755 0 0
devpts /dev/pts devpts rw,nosuid,noexec,relatime,gid=5,mode=620,ptmxmode=000 0 0
tmpfs /run tmpfs rw,nosuid,nodev,noexec,relatime,size=203520k,mode=755 0 0
/dev/vda5 / ext4 rw,relatime,errors=remount-ro 0 0
securityfs /sys/kernel/security securityfs rw,nosuid,nodev,noexec,relatime 0 0
tmpfs /dev/shm tmpfs rw,nosuid,nodev 0 0
tmpfs /run/lock tmpfs rw,nosuid,nodev,noexec,relatime,size=5120k 0 0
tmpfs /sys/fs/cgroup tmpfs ro,nosuid,nodev,noexec,mode=755 0 0
cgroup2 /sys/fs/cgroup/unified cgroup2 rw,nosuid,nodev,noexec,relatime,nsdelegate 0 0
cgroup /sys/fs/cgroup/systemd cgroup rw,nosuid,nodev,noexec,relatime,xattr,name=systemd 0 0
pstore /sys/fs/pstore pstore rw,nosuid,nodev,noexec,relatime 0 0
none /sys/fs/bpf bpf rw,nosuid,nodev,noexec,relatime,mode=700 0 0
cgroup /sys/fs/cgroup/blkio cgroup rw,nosuid,nodev,noexec,relatime,blkio 0 0
cgroup /sys/fs/cgroup/memory cgroup rw,nosuid,nodev,noexec,relatime,memory 0 0
cgroup /sys/fs/cgroup/hugetlb cgroup rw,nosuid,nodev,noexec,relatime,hugetlb 0 0
cgroup /sys/fs/cgroup/cpuset cgroup rw,nosuid,nodev,noexec,relatime,cpuset 0 0
cgroup /sys/fs/cgroup/cpu,cpuacct cgroup rw,nosuid,nodev,noexec,relatime,cpu,cpuacct 0 0
cgroup /sys/fs/cgroup/pids cgroup rw,nosuid,nodev,noexec,relatime,pids 0 0
cgroup /sys/fs/cgroup/rdma cgroup rw,nosuid,nodev,noexec,relatime,rdma 0 0
cgroup /sys/fs/cgroup/devices cgroup rw,nosuid,nodev,noexec,relatime,devices 0 0
cgroup /sys/fs/cgroup/freezer cgroup rw,nosuid,nodev,noexec,relatime,freezer 0 0
cgroup /sys/fs/cgroup/perf_event cgroup rw,nosuid,nodev,noexec,relatime,perf_event 0 0
systemd-1 /proc/sys/fs/binfmt_misc autofs rw,relatime,fd=28,pgrp=1,timeout=0,minproto=5,maxproto=5,direct,pipe_ino=14329 0 0
hugetlbfs /dev/hugepages hugetlbfs rw,relatime,pagesize=2M 0 0
mqueue /dev/mqueue mqueue rw,nosuid,nodev,noexec,relatime 0 0
debugfs /sys/kernel/debug debugfs rw,nosuid,nodev,noexec,relatime 0 0
tracefs /sys/kernel/tracing tracefs rw,nosuid,nodev,noexec,relatime 0 0
fusectl /sys/fs/fuse/connections fusectl rw,nosuid,nodev,noexec,relatime 0 0
configfs /sys/kernel/config configfs rw,nosuid,nodev,noexec,relatime 0 0
/dev/loop1 /snap/gnome-3-34-1804/24 squashfs ro,nodev,relatime 0 0
/dev/loop2 /snap/core18/1880 squashfs ro,nodev,relatime 0 0
/dev/loop3 /snap/gtk-common-themes/1506 squashfs ro,nodev,relatime 0 0
/dev/loop0 /snap/core18/1754 squashfs ro,nodev,relatime 0 0
/dev/loop5 /snap/snap-store/467 squashfs ro,nodev,relatime 0 0
/dev/loop6 /snap/gnome-3-34-1804/36 squashfs ro,nodev,relatime 0 0
/dev/loop7 /snap/snap-store/454 squashfs ro,nodev,relatime 0 0
/dev/loop8 /snap/snapd/8140 squashfs ro,nodev,relatime 0 0
/dev/vda1 /boot/efi vfat rw,relatime,fmask=0077,dmask=0077,codepage=437,iocharset=iso8859-1,shortname=mixed,errors=remount-ro 0 0
tmpfs /run/user/125 tmpfs rw,nosuid,nodev,relatime,size=203516k,mode=700,uid=125,gid=130 0 0
gvfsd-fuse /run/user/125/gvfs fuse.gvfsd-fuse rw,nosuid,nodev,relatime,user_id=125,group_id=130 0 0
/dev/loop9 /snap/snapd/8542 squashfs ro,nodev,relatime 0 0
tmpfs /run/user/1000 tmpfs rw,nosuid,nodev,relatime,size=203516k,mode=700,uid=1000,gid=1000 0 0
gvfsd-fuse /run/user/1000/gvfs fuse.gvfsd-fuse rw,nosuid,nodev,relatime,user_id=1000,group_id=1000 0 0
        "#;

        assert_eq!(find_net_cls_mount_inner(input), None)
    }
}
