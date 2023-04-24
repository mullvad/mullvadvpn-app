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

public class ShadowSocksProxy {
    private var proxyConfig: ProxyHandle
    private let remoteAddress: IPAddress
    private let remotePort: UInt16
    private let password: String
    private let cipher: String

    public init(remoteAddress: IPAddress, remotePort: UInt16, password: String, cipher: String) {
        proxyConfig = ProxyHandle(context: nil, port: 0)
        self.remoteAddress = remoteAddress
        self.remotePort = remotePort
        self.password = password
        self.cipher = cipher
    }

    /// The local port for the shadow socks proxy
    ///
    /// - Returns: The local port for the shadow socks proxy when it has started, 0 otherwise.
    public func localPort() -> UInt16 {
        proxyConfig.port
    }

    /// Starts the socks proxy
    public func start() {
        // Get the raw bytes of `addr.rawValue`
        remoteAddress.rawValue.withUnsafeBytes { unsafeAddressPointer in

            // Rebind the raw bytes to an array of bytes, and get a pointer to its beginning
            let rawAddr = unsafeAddressPointer.bindMemory(to: UInt8.self).baseAddress

            // Get the raw bytes access to `proxyConfig`
            _ = withUnsafeMutablePointer(to: &proxyConfig) { config in
                start_shadowsocks_proxy(
                    rawAddr,
                    UInt(remoteAddress.rawValue.count),
                    remotePort,
                    password,
                    UInt(password.count),
                    cipher,
                    UInt(cipher.count),
                    config
                )
            }
        }
    }

    /// Stops the socks proxy
    public func stop() {
        _ = withUnsafePointer(to: proxyConfig) { pointer in
            stop_shadowsocks_proxy(UnsafeMutablePointer(mutating: pointer))
        }
    }
}
