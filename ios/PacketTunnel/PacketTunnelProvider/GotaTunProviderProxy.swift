//
//  GotaTunProviderProxy.swift
//  PacketTunnel
//
//  Created by Emīls on 2026-08-13.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import NetworkExtension
import PacketTunnelCore

/// Abstracts away NEPacketTunnelProvider for GotaTun
/// Once we drop iOS17, this struct can become non-copyable too, which enforces the constraint that it should only be used by the GotaTun actor
struct GotaTunProviderProxy: TunnelProviderDelegate {
    /// `NEPacketTunnelProvider` predates Sendable, but the parts used here —
    /// the `reasserting` setter and `setTunnelNetworkSettings` — are callable
    /// from any thread.
    private nonisolated(unsafe) weak var provider: NEPacketTunnelProvider?

    init(provider: NEPacketTunnelProvider) {
        self.provider = provider
    }

    /// Scanning for the utun control socket reads no provider state, so it needs no isolation.
    nonisolated var tunnelFileDescriptor: Int32? {
        var ctlInfo = ctl_info()
        withUnsafeMutablePointer(to: &ctlInfo.ctl_name) {
            $0.withMemoryRebound(to: CChar.self, capacity: MemoryLayout.size(ofValue: $0.pointee)) {
                _ = strcpy($0, "com.apple.net.utun_control")
            }
        }
        for fd: Int32 in 0...1024 {
            var addr = sockaddr_ctl()
            var ret: Int32 = -1
            var len = socklen_t(MemoryLayout.size(ofValue: addr))
            withUnsafeMutablePointer(to: &addr) {
                $0.withMemoryRebound(to: sockaddr.self, capacity: 1) {
                    ret = getpeername(fd, $0, &len)
                }
            }
            if ret != 0 || addr.sc_family != AF_SYSTEM {
                continue
            }
            if ctlInfo.ctl_id == 0 {
                ret = ioctl(fd, CTLIOCGINFO, &ctlInfo)
                if ret != 0 {
                    continue
                }
            }
            if addr.sc_id == ctlInfo.ctl_id {
                return fd
            }
        }
        return nil
    }

    func reassertTunnel() async {
        guard let provider else { return }
        // Setting `reasserting` to `true` invalidates all sockets bound to the tunnel interface.
        provider.reasserting = true
        // Setting `reasserting` to `false` indicates that sockets can again be bound to the tunnel interface.
        // We'd prefer to keep the time interval between both as short as possible to prevent accidental leaks even when not using `includeAllNetworks`.
        provider.reasserting = false
    }

    func applyNetworkSettings(_ settings: TunnelInterfaceSettings) async throws {
        guard let provider else { return }
        try await provider.setTunnelNetworkSettings(settings.asTunnelSettings())
    }
}

// MARK: - Private socket structs for utun FD discovery
// Layout mirrors of the types in <sys/kern_control.h>, which the iOS SDK does not ship.
// The names intentionally match the C types.
// swift-format-ignore: TypeNamesShouldBeCapitalized
private struct ctl_info {
    var ctl_id: UInt32 = 0
    var ctl_name:
        (
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar,
            CChar, CChar, CChar, CChar, CChar, CChar, CChar, CChar
        ) = (
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0
        )
}

// swift-format-ignore: TypeNamesShouldBeCapitalized
private struct sockaddr_ctl {
    var sc_len: UInt8 = UInt8(MemoryLayout<sockaddr_ctl>.size)
    var sc_family: UInt8 = UInt8(AF_SYSTEM)
    var ss_sysaddr: UInt16 = 0
    var sc_id: UInt32 = 0
    var sc_unit: UInt32 = 0
    var sc_reserved: (UInt32, UInt32, UInt32, UInt32, UInt32) = (0, 0, 0, 0, 0)
}

private let CTLIOCGINFO: UInt = 0xC064_4E03
