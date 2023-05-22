//
//  ShadowSocksProxy.swift
//  MullvadREST
//
//  Created by Emils on 19/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import Shadowsocks

/// A Swift wrapper around a Rust implementation of Shadowsock
public class ShadowSocksProxy {
    private var proxyConfig: ProxyHandle
    private let forwardAddress: IPAddress
    private let forwardPort: UInt16
    private let bridgeIPAddress: IPAddress
    private let bridgeIPPort: UInt16
    private let password: String
    private let cipher: String
    private var didStart = false
    private let stateLock = NSLock()

    public init(
        forwardAddress: IPAddress,
        forwardPort: UInt16,
        bridgeIPAddress: IPAddress,
        bridgePort: UInt16,
        password: String,
        cipher: String
    ) {
        self.proxyConfig = ProxyHandle(context: nil, port: 0)
        self.forwardAddress = forwardAddress
        self.forwardPort = forwardPort
        self.bridgeIPAddress = bridgeIPAddress
        self.bridgeIPPort = bridgePort
        self.password = password
        self.cipher = cipher
    }

    /// The local port for the shadow socks proxy
    ///
    /// - Returns: The local port for the shadow socks proxy when it has started, 0 otherwise.
    public func localPort() -> UInt16 {
        stateLock.lock()
        defer { stateLock.unlock() }
        return proxyConfig.port
    }

    deinit {
        stop()
    }

    /// Starts the socks proxy
    public func start() {
        stateLock.lock()
        defer { stateLock.unlock() }
        guard didStart == false else { return }
        didStart = true

        // Get the raw bytes access to `proxyConfig`
        _ = withUnsafeMutablePointer(to: &proxyConfig) { config in
            start_shadowsocks_proxy(
                (forwardAddress.rawValue as NSData).bytes,
                UInt(forwardAddress.rawValue.count),
                forwardPort,
                (bridgeIPAddress.rawValue as NSData).bytes,
                UInt(bridgeIPAddress.rawValue.count),
                bridgeIPPort,
                password,
                UInt(password.count),
                cipher,
                UInt(cipher.count),
                config
            )
        }
    }

    /// Stops the socks proxy
    public func stop() {
        stateLock.lock()
        defer { stateLock.unlock() }
        guard didStart == true else { return }
        didStart = false

        _ = withUnsafePointer(to: proxyConfig) { pointer in
            stop_shadowsocks_proxy(UnsafeMutablePointer(mutating: pointer))
        }
    }
}
