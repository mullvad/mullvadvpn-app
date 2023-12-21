//
//  PersistentAccessMethod.swift
//  MullvadVPN
//
//  Created by pronebird on 15/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import Network

/// Persistent access method model.
public struct PersistentAccessMethod: Identifiable, Codable, Equatable {
    /// The unique identifier used for referencing the access method entry in a persistent store.
    public var id: UUID

    /// The user-defined name for access method.
    public var name: String

    /// The flag indicating whether configuration is enabled.
    public var isEnabled: Bool

    /// Proxy configuration.
    public var proxyConfiguration: PersistentProxyConfiguration

    public init(id: UUID, name: String, isEnabled: Bool, proxyConfiguration: PersistentProxyConfiguration) {
        self.id = id
        self.name = name
        self.isEnabled = isEnabled
        self.proxyConfiguration = proxyConfiguration
    }

    public static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.id == rhs.id
    }
}

/// Persistent proxy configuration.
public enum PersistentProxyConfiguration: Codable {
    /// Direct communication without proxy.
    case direct

    /// Communication over bridges.
    case bridges

    /// Communication over shadowsocks.
    case shadowsocks(ShadowsocksConfiguration)

    /// Communication over socks5 proxy.
    case socks5(SocksConfiguration)
}

extension PersistentProxyConfiguration {
    /// Socks autentication method.
    public enum SocksAuthentication: Codable {
        case noAuthentication
        case usernamePassword(username: String, password: String)
    }

    /// Socks v5 proxy configuration.
    public struct SocksConfiguration: Codable {
        /// Proxy server address.
        public var server: AnyIPAddress

        /// Proxy server port.
        public var port: UInt16

        /// Authentication method.
        public var authentication: SocksAuthentication

        public init(server: AnyIPAddress, port: UInt16, authentication: SocksAuthentication) {
            self.server = server
            self.port = port
            self.authentication = authentication
        }
    }

    /// Shadowsocks configuration.
    public struct ShadowsocksConfiguration: Codable {
        /// Server address.
        public var server: AnyIPAddress

        /// Server port.
        public var port: UInt16

        /// Server password.
        public var password: String

        /// Server cipher.
        public var cipher: ShadowsocksCipherOptions

        public init(server: AnyIPAddress, port: UInt16, password: String, cipher: ShadowsocksCipherOptions) {
            self.server = server
            self.port = port
            self.password = password
            self.cipher = cipher
        }
    }
}
