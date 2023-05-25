//
//  ShadowsocksProxy.swift
//  MullvadREST
//
//  Created by Emils on 19/04/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Network
import Shadowsocks

/// A Swift wrapper around a Rust implementation of Shadowsocks proxy instance
public class ShadowsocksProxy {
    private var proxyConfig: ProxyHandle
    private let forwardAddress: IPAddress
    private let forwardPort: UInt16
    private let bridgeAddress: IPAddress
    private let bridgePort: UInt16
    private let password: String
    private let cipher: String
    private var didStart = false
    private let stateLock = NSLock()

    public init(
        forwardAddress: IPAddress,
        forwardPort: UInt16,
        bridgeAddress: IPAddress,
        bridgePort: UInt16,
        password: String,
        cipher: String
    ) {
        proxyConfig = ProxyHandle(context: nil, port: 0)
        self.forwardAddress = forwardAddress
        self.forwardPort = forwardPort
        self.bridgeAddress = bridgeAddress
        self.bridgePort = bridgePort
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
                forwardAddress.rawValue.map { $0 },
                UInt(forwardAddress.rawValue.count),
                forwardPort,
                bridgeAddress.rawValue.map { $0 },
                UInt(bridgeAddress.rawValue.count),
                bridgePort,
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

        _ = withUnsafeMutablePointer(to: &proxyConfig) { config in
            stop_shadowsocks_proxy(config)
        }
    }
}
