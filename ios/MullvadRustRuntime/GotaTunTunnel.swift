//
//  GotaTunTunnel.swift
//  MullvadRustRuntime
//
//  Created by Mullvad VPN.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

// MARK: - Config builder

/// Builder for GotaTun tunnel configuration.
/// Wraps the Rust FFI config builder pattern.
public final class GotaTunConfigBuilder {
    private var handle: OpaquePointer?

    /// Create a new config builder with the required private key and tunnel IPv4 address.
    /// - Parameters:
    ///   - privateKey: WireGuard private key (32 bytes).
    ///   - ipv4Address: Tunnel interface IPv4 address (e.g. "10.64.0.2").
    public init?(privateKey: Data, ipv4Address: String) {
        guard privateKey.count >= 32 else { return nil }

        let ptr = privateKey.withUnsafeBytes { keyPtr -> OpaquePointer? in
            ipv4Address.withCString { addrPtr in
                gotatun_config_new(
                    keyPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                    addrPtr
                )
            }
        }

        guard let ptr else { return nil }
        handle = ptr
    }

    deinit {
        if let handle {
            gotatun_config_free(handle)
        }
    }

    /// Set the tunnel MTU.
    @discardableResult
    public func setMTU(_ mtu: UInt16) -> Self {
        if let handle {
            gotatun_config_set_mtu(handle, mtu)
        }
        return self
    }

    /// Set the exit peer.
    /// - Parameters:
    ///   - publicKey: Peer's WireGuard public key (32 bytes).
    ///   - endpoint: Peer's endpoint as "ip:port".
    @discardableResult
    public func setExitPeer(publicKey: Data, endpoint: String) -> Self {
        guard let handle, publicKey.count >= 32 else { return self }

        publicKey.withUnsafeBytes { keyPtr in
            endpoint.withCString { endpointPtr in
                gotatun_config_set_exit_peer(
                    handle,
                    keyPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                    endpointPtr
                )
            }
        }
        return self
    }

    /// Set the gateway IPv4 address used for connectivity pings.
    @discardableResult
    public func setGateway(_ gateway: String) -> Self {
        if let handle {
            gateway.withCString { ptr in
                gotatun_config_set_gateway(handle, ptr)
            }
        }
        return self
    }

    /// Set the retry attempt number.
    @discardableResult
    public func setRetryAttempt(_ attempt: UInt32) -> Self {
        if let handle {
            gotatun_config_set_retry_attempt(handle, attempt)
        }
        return self
    }

    /// Set the entry peer for multihop.
    @discardableResult
    public func setEntryPeer(publicKey: Data, endpoint: String) -> Self {
        guard let handle, publicKey.count >= 32 else { return self }

        publicKey.withUnsafeBytes { keyPtr in
            endpoint.withCString { endpointPtr in
                gotatun_config_set_entry_peer(
                    handle,
                    keyPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                    endpointPtr
                )
            }
        }
        return self
    }

    /// Enable post-quantum key exchange.
    @discardableResult
    public func enablePostQuantum() -> Self {
        if let handle {
            gotatun_config_enable_pq(handle)
        }
        return self
    }

    /// Set obfuscation to UDP-over-TCP.
    @discardableResult
    public func setObfuscationUdpOverTcp() -> Self {
        if let handle {
            gotatun_config_set_obfuscation_udp_over_tcp(handle)
        }
        return self
    }

    /// Set obfuscation to Shadowsocks.
    @discardableResult
    public func setObfuscationShadowsocks() -> Self {
        if let handle {
            gotatun_config_set_obfuscation_shadowsocks(handle)
        }
        return self
    }

    /// Set obfuscation to QUIC.
    @discardableResult
    public func setObfuscationQuic(hostname: String, token: String) -> Self {
        if let handle {
            hostname.withCString { hostnamePtr in
                token.withCString { tokenPtr in
                    gotatun_config_set_obfuscation_quic(handle, hostnamePtr, tokenPtr)
                }
            }
        }
        return self
    }

    /// Set obfuscation to LWO (Lightweight Obfuscation).
    @discardableResult
    public func setObfuscationLwo(clientPublicKey: Data, serverPublicKey: Data) -> Self {
        guard let handle, clientPublicKey.count >= 32, serverPublicKey.count >= 32 else { return self }
        clientPublicKey.withUnsafeBytes { clientPtr in
            serverPublicKey.withUnsafeBytes { serverPtr in
                gotatun_config_set_obfuscation_lwo(
                    handle,
                    clientPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                    serverPtr.baseAddress?.assumingMemoryBound(to: UInt8.self)
                )
            }
        }
        return self
    }

    /// Enable DAITA.
    @discardableResult
    public func enableDAITA() -> Self {
        if let handle {
            gotatun_config_enable_daita(handle)
        }
        return self
    }

    /// Set the establish timeout in seconds.
    @discardableResult
    public func setEstablishTimeout(_ timeoutSecs: UInt32) -> Self {
        if let handle {
            gotatun_config_set_establish_timeout(handle, timeoutSecs)
        }
        return self
    }

    /// Consume the builder and return the raw config handle.
    /// After calling this, the builder is invalidated and must not be used.
    func consumeHandle() -> OpaquePointer? {
        let h = handle
        handle = nil
        return h
    }
}

// MARK: - Tunnel handle

/// Manages a running GotaTun tunnel instance.
/// Wraps the Rust FFI tunnel lifecycle.
public final class GotaTunTunnel: @unchecked Sendable {
    private var handle: OpaquePointer?
    private let lock = NSLock()

    /// Weak reference prevents the callback closure from preventing deallocation
    private var callbackRetainer: AnyObject?

    /// Start a tunnel with the given config and TUN file descriptor.
    /// - Parameters:
    ///   - tunnelFd: The TUN device file descriptor from the packet tunnel extension.
    ///   - config: A config builder (consumed by this call).
    ///   - onConnected: Called when the tunnel is up and traffic flows.
    ///   - onTimeout: Called when the pinger times out.
    ///   - onError: Called on fatal error with a description string.
    /// - Returns: A tunnel instance, or nil if startup failed.
    public static func start(
        tunnelFd: Int32,
        config: GotaTunConfigBuilder,
        onConnected: @escaping () -> Void,
        onTimeout: @escaping () -> Void,
        onError: @escaping (String) -> Void
    ) -> GotaTunTunnel? {
        guard let configHandle = config.consumeHandle() else {
            return nil
        }

        let tunnel = GotaTunTunnel()

        let box = CallbackBox(
            onConnected: onConnected,
            onTimeout: onTimeout,
            onError: onError
        )
        tunnel.callbackRetainer = box
        let context = Unmanaged.passRetained(box).toOpaque()

        let callbacks = GotaTunCallbacks(
            context: context,
            on_connected: { ctx in
                guard let ctx else { return }
                Unmanaged<CallbackBox>.fromOpaque(ctx).takeUnretainedValue().onConnected()
            },
            on_timeout: { ctx in
                guard let ctx else { return }
                Unmanaged<CallbackBox>.fromOpaque(ctx).takeUnretainedValue().onTimeout()
            },
            on_error: { ctx, msg in
                guard let ctx else { return }
                let message = msg.map { String(cString: $0) } ?? "Unknown error"
                Unmanaged<CallbackBox>.fromOpaque(ctx).takeUnretainedValue().onError(message)
            }
        )

        let handle = gotatun_start_tunnel(tunnelFd, configHandle, callbacks)
        guard let handle else {
            Unmanaged<CallbackBox>.fromOpaque(context).release()
            tunnel.callbackRetainer = nil
            return nil
        }

        tunnel.handle = handle
        return tunnel
    }

    /// Stop the tunnel. Safe to call multiple times.
    public func stop() {
        lock.withLock {
            guard let handle else { return }
            gotatun_stop_tunnel(handle)
            self.handle = nil
            callbackRetainer = nil
        }
    }

    /// Recycle UDP sockets after a network path change.
    public func recycleUdpSockets() {
        lock.withLock {
            guard let handle else { return }
            gotatun_recycle_sockets(handle)
        }
    }

    /// Suspend the tunnel (device sleep).
    public func suspend() {
        lock.withLock {
            guard let handle else { return }
            gotatun_suspend_tunnel(handle)
        }
    }

    /// Wake the tunnel from suspension.
    public func wake() {
        lock.withLock {
            guard let handle else { return }
            gotatun_wake_tunnel(handle)
        }
    }

    deinit {
        stop()
    }
}

// MARK: - Callback box

private final class CallbackBox {
    let onConnected: () -> Void
    let onTimeout: () -> Void
    let onError: (String) -> Void

    init(
        onConnected: @escaping () -> Void,
        onTimeout: @escaping () -> Void,
        onError: @escaping (String) -> Void
    ) {
        self.onConnected = onConnected
        self.onTimeout = onTimeout
        self.onError = onError
    }
}
