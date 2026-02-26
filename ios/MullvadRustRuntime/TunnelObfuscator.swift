//
//  TunnelObfuscator.swift
//  TunnelObfuscation
//
//  Created by pronebird on 19/06/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntimeProxy
import MullvadTypes
import Network
import WireGuardKitTypes

public enum TunnelObfuscationProtocol {
    case udpOverTcp
    case shadowsocks
    case quic(hostname: String, token: String)
    case lwo(serverPublicKey: PublicKey)
}

public protocol TunnelObfuscation {
    init(
        remoteAddress: IPAddress,
        remotePort: UInt16,
        obfuscationProtocol: TunnelObfuscationProtocol,
        clientPublicKey: PublicKey
    )
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
    private let port: UInt16
    private let obfuscationProtocol: TunnelObfuscationProtocol
    private let clientPublicKey: PublicKey

    private var proxyHandle = ProxyHandle(context: nil, port: 0)
    private var isStarted = false

    /// Returns local UDP port used by proxy and bound to 127.0.0.1 (IPv4).
    /// The returned value can be zero if obfuscator hasn't started yet.
    public var localUdpPort: UInt16 {
        return stateLock.withLock { proxyHandle.port }
    }

    public var remotePort: UInt16 { port }

    public var transportLayer: TransportLayer {
        switch obfuscationProtocol {
        case .udpOverTcp:
            .tcp
        case .shadowsocks, .quic, .lwo:
            .udp
        }
    }

    /// Initialize tunnel obfuscator with remote server address and port where obfuscation is running.
    public init(
        remoteAddress: IPAddress,
        remotePort: UInt16,
        obfuscationProtocol: TunnelObfuscationProtocol,
        clientPublicKey: PublicKey
    ) {
        self.remoteAddress = remoteAddress
        self.port = remotePort
        self.obfuscationProtocol = obfuscationProtocol
        self.clientPublicKey = clientPublicKey
    }

    deinit {
        stop()
    }

    public func start() {
        stateLock.withLock {
            guard !isStarted else { return }

            let result = withUnsafeMutablePointer(to: &proxyHandle) { proxyHandlePointer in
                let addressData = remoteAddress.rawValue

                return switch obfuscationProtocol {
                case .udpOverTcp:
                    start_udp2tcp_obfuscator_proxy(
                        addressData.map { $0 },
                        UInt(addressData.count),
                        port,
                        proxyHandlePointer
                    )
                case .shadowsocks:
                    start_shadowsocks_obfuscator_proxy(
                        addressData.map { $0 },
                        UInt(addressData.count),
                        port,
                        proxyHandlePointer
                    )
                case let .quic(hostname, token):
                    start_quic_obfuscator_proxy(
                        addressData.map { $0 },
                        UInt(addressData.count),
                        port,
                        hostname,
                        token,
                        proxyHandlePointer
                    )
                case let .lwo(serverPublicKey):
                    clientPublicKey.rawValue.withUnsafeBytes { clientKeyPtr in
                        serverPublicKey.rawValue.withUnsafeBytes { serverKeyPtr in
                            start_lwo_obfuscator_proxy(
                                addressData.map { $0 },
                                UInt(addressData.count),
                                port,
                                clientKeyPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                                serverKeyPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                                proxyHandlePointer
                            )
                        }
                    }
                }
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
