//
//  RustGotaTunAdapter.swift
//  PacketTunnel
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadRustRuntime
import PacketTunnelCore

/// GotaTun adapter backed by the Rust FFI via the UniFFI-generated `GotaTunTunnel`.
///
/// This is the single boundary that maps `PacketTunnelCore`'s FFI-agnostic
/// `GotaTunConfig` to the UniFFI-generated `MullvadRustRuntime.GotaTunConfig` and
/// bridges the generated `GotaTunCallback` to the domain `GotaTunCallbackHandler`,
/// keeping `PacketTunnelCore` decoupled from the FFI layer.
/// Each instance manages a single connection attempt.
final class RustGotaTunAdapter: GotaTunAdapterProtocol, @unchecked Sendable {
    private let logger = Logger(label: "RustGotaTunAdapter")
    private var tunnel: GotaTunTunnel?

    func startTunnel(config: PacketTunnelCore.GotaTunConfig, callbackHandler: GotaTunCallbackHandler) throws {
        let ffiConfig = Self.makeFfiConfig(config)

        do {
            tunnel = try GotaTunTunnel.start(
                tunFd: config.tunnelFd,
                config: ffiConfig,
                callback: CallbackProxy(callbackHandler)
            )
        } catch let error as GotaTunFfiError {
            throw Self.mapError(error)
        }

        logger.debug("Tunnel started via GotaTunTunnel")
    }

    func stopTunnel() {
        tunnel?.stop()
        tunnel = nil
    }

    func recycleUdpSockets() {
        tunnel?.recycleUdpSockets()
    }

    func suspendTunnel() {
        tunnel?.suspend()
    }

    func wakeTunnel() {
        tunnel?.wake()
    }

    deinit {
        stopTunnel()
    }

    // MARK: - Mapping

    /// Build the UniFFI config from the domain config.
    private static func makeFfiConfig(_ config: PacketTunnelCore.GotaTunConfig) -> MullvadRustRuntime.GotaTunConfig {
        let exitPeer = GotaTunPeer(
            publicKey: config.exitPeerPublicKey,
            endpoint: config.exitPeerEndpoint
        )

        let entryPeer: GotaTunPeer? = {
            guard let key = config.entryPeerPublicKey, let endpoint = config.entryPeerEndpoint
            else { return nil }
            return GotaTunPeer(publicKey: key, endpoint: endpoint)
        }()

        return MullvadRustRuntime.GotaTunConfig(
            privateKey: config.privateKey,
            ipv4Address: "\(config.ipv4Address)",
            ipv6Address: "\(config.ipv6Address)",
            mtu: config.mtu,
            exitPeer: exitPeer,
            entryPeer: entryPeer,
            ipv4Gateway: config.ipv4Gateway,
            establishTimeoutSecs: config.establishTimeout,
            enablePq: config.isPostQuantum,
            enableDaita: config.isDaitaEnabled,
            obfuscation: makeObfuscation(config)
        )
    }

    private static func makeObfuscation(_ config: PacketTunnelCore.GotaTunConfig) -> GotaTunObfuscation {
        switch config.obfuscationMethod {
        case .off:
            return .off
        case .udpOverTcp:
            return .udpOverTcp
        case .shadowsocks:
            return .shadowsocks
        case let .quic(hostname, token):
            return .quic(hostname: hostname, token: token)
        case .lwo:
            // LWO obfuscates the handshake with the ingress relay's key.
            let ingressPeerKey = config.entryPeerPublicKey ?? config.exitPeerPublicKey
            return .lwo(clientPublicKey: config.clientPublicKey, serverPublicKey: ingressPeerKey)
        }
    }

    private static func mapError(_ error: GotaTunFfiError) -> GotaTunError {
        switch error {
        case let .InvalidConfig(message):
            return .invalidConfig(message)
        case let .Internal(message):
            return .internalError(message)
        }
    }
}

/// Bridges the UniFFI-generated `GotaTunCallback` to the domain `GotaTunCallbackHandler`.
private final class CallbackProxy: GotaTunCallback {
    private let handler: GotaTunCallbackHandler

    init(_ handler: GotaTunCallbackHandler) {
        self.handler = handler
    }

    func onConnected() {
        handler.onConnected()
    }

    func onTimeout() {
        handler.onTimeout()
    }

    func onError(message: String) {
        handler.onError(.internalError(message))
    }
}
