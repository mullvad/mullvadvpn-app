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

/// GotaTun adapter backed by the Rust FFI via `GotaTunTunnel`.
/// Each instance manages a single connection attempt.
final class RustGotaTunAdapter: GotaTunAdapterProtocol, @unchecked Sendable {
    private let logger = Logger(label: "RustGotaTunAdapter")
    private var tunnel: GotaTunTunnel?

    func startTunnel(config: GotaTunConfig, callbackHandler: GotaTunCallbackHandler) throws {
        // Build config using the builder API
        let ipv4String =
            config.interfaceAddresses
            .first { $0.contains(".") }?
            .components(separatedBy: "/").first ?? "10.64.0.2"

        guard
            let builder = GotaTunConfigBuilder(
                privateKey: config.privateKey,
                ipv4Address: ipv4String
            )
        else {
            throw GotaTunError.invalidConfig("Failed to create config builder")
        }

        builder
            .setMTU(config.mtu)
            .setExitPeer(
                publicKey: config.exitPeerPublicKey,
                endpoint: config.exitPeerEndpoint
            )

        // Set entry peer if multihop
        if let entryKey = config.entryPeerPublicKey,
            let entryEndpoint = config.entryPeerEndpoint
        {
            builder.setEntryPeer(publicKey: entryKey, endpoint: entryEndpoint)
        }

        if config.isPostQuantum {
            builder.enablePostQuantum()
        }
        if config.isDaitaEnabled {
            builder.enableDAITA()
        }

        // Set obfuscation
        switch config.obfuscationMethod {
        case .udpOverTcp:
            builder.setObfuscationUdpOverTcp()
        case .shadowsocks:
            builder.setObfuscationShadowsocks()
        case let .quic(hostname, token):
            builder.setObfuscationQuic(hostname: hostname, token: token)
        case .lwo:
            let ingressPeerKey = config.entryPeerPublicKey ?? config.exitPeerPublicKey
            builder.setObfuscationLwo(
                clientPublicKey: config.clientPublicKey,
                serverPublicKey: ingressPeerKey
            )
        case .off:
            break
        }

        builder
            .setGateway(config.ipv4Gateway)
            .setEstablishTimeout(config.establishTimeout)

        // Start tunnel
        guard
            let started = GotaTunTunnel.start(
                tunnelFd: config.tunnelFd,
                config: builder,
                onConnected: { [weak callbackHandler] in
                    callbackHandler?.onConnected()
                },
                onTimeout: { [weak callbackHandler] in
                    callbackHandler?.onTimeout()
                },
                onError: { [weak callbackHandler] message in
                    callbackHandler?.onError(GotaTunError.internalError(message))
                }
            )
        else {
            throw GotaTunError.invalidConfig("gotatun_start_tunnel returned null")
        }

        tunnel = started
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
}
