//
//  PersistentProxyConfiguration.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-07-31.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

/// Persistent proxy configuration; formerly contained in PersistentAccessMethod.swift.
public enum PersistentProxyConfiguration: Codable, Equatable, Sendable {
    /// Direct communication without proxy.
    case direct

    /// Communication over bridges.
    case bridges

    /// Communication over proxy address from a DNS.
    case encryptedDNS

    /// Communication over shadowsocks.
    case shadowsocks(ShadowsocksConfiguration)

    /// Communication over socks5 proxy.
    case socks5(SocksConfiguration)
}

extension PersistentProxyConfiguration {
    /// Socks autentication method.
    public enum SocksAuthentication: Codable, Equatable, Sendable {
        case noAuthentication
        case authentication(UserCredential)
    }

    public struct UserCredential: Codable, Equatable, Sendable {
        public let username: String
        public let password: String

        public init(username: String, password: String) {
            self.username = username
            self.password = password
        }
    }

    /// Socks v5 proxy configuration.
    public struct SocksConfiguration: Codable, Equatable, Sendable {
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

        public var credential: UserCredential? {
            guard case let .authentication(credential) = authentication else {
                return nil
            }
            return credential
        }

        public var toAnyIPEndpoint: AnyIPEndpoint {
            switch server {
            case let .ipv4(ip):
                return .ipv4(IPv4Endpoint(ip: ip, port: port))
            case let .ipv6(ip):
                return .ipv6(IPv6Endpoint(ip: ip, port: port))
            }
        }
    }

    /// Shadowsocks configuration.
    public struct ShadowsocksConfiguration: Codable, Equatable, Sendable {
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
