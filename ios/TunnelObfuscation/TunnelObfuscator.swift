//
//  TunnelObfuscator.swift
//  TunnelObfuscation
//
//  Created by pronebird on 19/06/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import TunnelObfuscatorProxy

/// Class that implements UDP over TCP obfuscation by accepting traffic on a local UDP port and proxying it over TCP to the remote endpoint.
public final class TunnelObfuscator {
    private let stateLock = NSLock()
    private let remoteAddress: IPAddress
    private let tcpPort: UInt16

    private var proxyHandle = ProxyHandle(context: nil, port: 0)
    private var isStarted = false

    /// Returns local UDP port used by proxy and bound to 127.0.0.1 (IPv4).
    /// The returned value can be zero if obfuscator hasn't started yet.
    public var localUdpPort: UInt16 {
        return stateLock.withLock { proxyHandle.port }
    }

    /// Initialize tunnel obfuscator with remote server address and TCP port where udp2tcp is running.
    public init(remoteAddress: IPAddress, tcpPort: UInt16) {
        self.remoteAddress = remoteAddress
        self.tcpPort = tcpPort
    }

    deinit {
        stop()
    }

    public func start() {
        stateLock.withLock {
            guard !isStarted else { return }

            let result = withUnsafeMutablePointer(to: &proxyHandle) { proxyHandlePointer in
                let addressData = remoteAddress.rawValue

                return start_tunnel_obfuscator_proxy(
                    addressData.map { $0 },
                    UInt(addressData.count),
                    tcpPort,
                    proxyHandlePointer
                )
            }

            assert(result == 0)

            isStarted = true
        }
    }

    public func stop() {
        stateLock.withLock {
            guard isStarted else { return }

            let result = withUnsafeMutablePointer(to: &proxyHandle) {
                stop_tunnel_obfuscator_proxy($0)
            }

            assert(result == 0)

            isStarted = false
        }
    }
}
