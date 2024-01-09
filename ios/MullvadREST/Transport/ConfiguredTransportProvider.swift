//
//  ConfiguredTransportProvider.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2024-01-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

/// Allows the creation of `RESTTransport` objects that bypass the network routing logic provided by `TransportProvider`.
public class ConfiguredTransportProvider {
    private let shadowsocksLoader: ShadowsocksLoaderProtocol
    private let addressCache: REST.AddressCache

    public init(shadowsocksLoader: ShadowsocksLoaderProtocol, addressCache: REST.AddressCache) {
        self.shadowsocksLoader = shadowsocksLoader
        self.addressCache = addressCache
    }

    public func makeTransport(with configuration: PersistentProxyConfiguration) throws -> RESTTransport {
        let urlSession = REST.makeURLSession()
        switch configuration {
        case .direct:
            return URLSessionTransport(urlSession: urlSession)
        case .bridges:
            let shadowsocksConfiguration = try shadowsocksLoader.load()
            return ShadowsocksTransport(
                urlSession: urlSession,
                configuration: shadowsocksConfiguration,
                addressCache: addressCache
            )
        case let .shadowsocks(shadowSocksConfiguration):
            return ShadowsocksTransport(
                urlSession: urlSession,
                configuration: ShadowsocksConfiguration(
                    address: shadowSocksConfiguration.server,
                    port: shadowSocksConfiguration.port,
                    password: shadowSocksConfiguration.password,
                    cipher: shadowSocksConfiguration.cipher.rawValue.description
                ),
                addressCache: addressCache
            )
        case let .socks5(socksConfiguration):
            var maybeUsername: String?
            var maybePassword: String?
            if case let .usernamePassword(username, password) = socksConfiguration.authentication {
                maybeUsername = username
                maybePassword = password
            }
            return URLSessionSocks5Transport(
                urlSession: urlSession,
                configuration: Socks5Configuration(
                    proxyEndpoint: socksConfiguration.toAnyIPEndpoint,
                    username: maybeUsername,
                    password: maybePassword
                ),
                addressCache: addressCache
            )
        }
    }
}
