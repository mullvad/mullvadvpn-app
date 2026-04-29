//
//  GotaTunAdapterProtocol.swift
//  PacketTunnelCore
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// Error reported by the GotaTun Rust adapter during tunnel setup.
/// Each variant maps to a non-recoverable `BlockedStateReason`.
public enum GotaTunError: Error, Sendable, Equatable {
    case ephemeralPeerNegotiation(String)
    case obfuscationSetup(String)
    case invalidConfig(String)
    case internalError(String)
}

/// Callback handler invoked by the GotaTun adapter (Rust → Swift).
/// All methods are called at most once per adapter instance.
/// Implementations must be `Sendable` since callbacks arrive from Rust-owned threads.
public protocol GotaTunCallbackHandler: AnyObject, Sendable {
    /// Tunnel is up, pinger confirmed traffic flows.
    func onConnected()

    /// Pinger timed out — either during initial setup or after `onConnected`.
    func onTimeout()

    /// Fatal error during setup.
    func onError(_ error: GotaTunError)
}

/// Placeholder config type for tunnel setup.
/// Will be replaced by UniFFI-generated `GotaTunConfig` in Phase 2.
public struct GotaTunConfig: Sendable {
    /// File descriptor for the TUN device provided by the packet tunnel extension.
    public let tunnelFd: Int32
    public let privateKey: Data
    public let interfaceAddresses: [String]
    public let mtu: UInt16
    /// IPv4 gateway address for the tunnel (e.g. "10.64.0.1").
    public let ipv4Gateway: String
    /// Client (device) public key — derived from privateKey.
    public let clientPublicKey: Data
    public let exitPeerPublicKey: Data
    public let exitPeerEndpoint: String
    public let entryPeerPublicKey: Data?
    public let entryPeerEndpoint: String?
    public let isPostQuantum: Bool
    public let isDaitaEnabled: Bool
    /// How long to wait for the tunnel to establish connectivity (seconds).
    public let establishTimeout: UInt32
    /// Obfuscation method for the ingress relay.
    public let obfuscationMethod: ObfuscationMethod
}

/// Each instance represents a single tunnel connection attempt.
/// Create a new instance for each retry.
///
/// Internally, Rust may create multiple GotaTun `Device` instances during a
/// single connection attempt (e.g., smoltcp device for PQ negotiation, then
/// final tunnel device). This protocol only exposes the outer connection.
///
/// ## Concurrency Safety
/// All methods are safe to call from any thread. The adapter uses internal
/// synchronization to handle races:
/// - Once a terminal callback has fired, all subsequent calls are no-ops.
/// - `stopTunnel()` suppresses pending callbacks.
public protocol GotaTunAdapterProtocol: Sendable {
    /// Start the tunnel. Non-blocking.
    /// Exactly one of `onConnected`/`onTimeout`/`onError` will be called.
    /// After `onConnected`, `onTimeout` may fire once more if connection is lost.
    func startTunnel(config: GotaTunConfig, callbackHandler: GotaTunCallbackHandler) throws

    /// Stop the tunnel. Safe to call multiple times or concurrently with callbacks.
    /// After this returns, no callbacks will fire.
    func stopTunnel()

    /// Recycle UDP sockets when NWPath changes (e.g. WiFi → cellular).
    /// No-op if tunnel has terminated or is still in PQ negotiation.
    func recycleUdpSockets()

    /// Suspend tunnel (device sleep). Pauses pinger if connected.
    /// Abandons PQ negotiation if in progress — `onTimeout` will fire.
    func suspendTunnel()

    /// Wake tunnel from suspension. Resumes pinger.
    func wakeTunnel()
}

/// Factory for creating adapter instances. One instance per connection attempt.
public protocol GotaTunAdapterFactory: Sendable {
    func makeAdapter() -> GotaTunAdapterProtocol
}
