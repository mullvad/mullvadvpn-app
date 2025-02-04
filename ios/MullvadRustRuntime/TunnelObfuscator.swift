//
//  TunnelObfuscator.swift
//  TunnelObfuscation
//
//  Created by pronebird on 19/06/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntimeProxy
import MullvadTypes
import Network

public enum TunnelObfuscationProtocol {
    case udpOverTcp
    case shadowsocks
}

public protocol TunnelObfuscation {
    init(remoteAddress: IPAddress, tcpPort: UInt16, obfuscationProtocol: TunnelObfuscationProtocol)
    func start()
    func stop()
    var localUdpPort: UInt16 { get }
    var remotePort: UInt16 { get }

    var transportLayer: TransportLayer { get }
}

/// Class that implements obfuscation by accepting traffic on a local port and proxying it to the remote endpoint.
///
/// The obfuscation happens either by wrapping UDP traffic into TCP traffic, or by using a local shadowsocks server
/// to encrypt the UDP traffic sent.
public final class TunnelObfuscator: TunnelObfuscation {
    private let stateLock = NSLock()
    private let remoteAddress: IPAddress
    internal let tcpPort: UInt16
    internal let obfuscationProtocol: TunnelObfuscationProtocol

    private var proxyHandle = ProxyHandle(context: nil, port: 0)
    private var isStarted = false

    /// Returns local UDP port used by proxy and bound to 127.0.0.1 (IPv4).
    /// The returned value can be zero if obfuscator hasn't started yet.
    public var localUdpPort: UInt16 {
        return stateLock.withLock { proxyHandle.port }
    }

    public var remotePort: UInt16 { tcpPort }

    public var transportLayer: TransportLayer {
        switch obfuscationProtocol {
        case .udpOverTcp:
            .tcp
        case .shadowsocks:
            .udp
        }
    }

    /// Initialize tunnel obfuscator with remote server address and TCP port where udp2tcp is running.
    public init(remoteAddress: IPAddress, tcpPort: UInt16, obfuscationProtocol: TunnelObfuscationProtocol) {
        self.remoteAddress = remoteAddress
        self.tcpPort = tcpPort
        self.obfuscationProtocol = obfuscationProtocol
    }

    deinit {
        stop()
    }

    public func start() {
        stateLock.withLock {
            guard !isStarted else { return }

            let obfuscationProtocol = switch obfuscationProtocol {
            case .udpOverTcp: TunnelObfuscatorProtocol(0)
            case .shadowsocks: TunnelObfuscatorProtocol(1)
            }

            let result = withUnsafeMutablePointer(to: &proxyHandle) { proxyHandlePointer in
                let addressData = remoteAddress.rawValue

                return start_tunnel_obfuscator_proxy(
                    addressData.map { $0 },
                    UInt(addressData.count),
                    tcpPort,
                    obfuscationProtocol,
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
