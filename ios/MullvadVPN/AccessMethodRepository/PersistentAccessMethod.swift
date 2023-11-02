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
struct PersistentAccessMethod: Identifiable, Codable {
    /// The unique identifier used for referencing the access method entry in a persistent store.
    var id: UUID

    /// The user-defined name for access method.
    var name: String

    /// The flag indicating whether configuration is enabled.
    var isEnabled: Bool

    /// Proxy configuration.
    var proxyConfiguration: PersistentProxyConfiguration
}

/// Persistent proxy configuration.
enum PersistentProxyConfiguration: Codable {
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
    enum SocksAuthentication: Codable {
        case noAuthentication
        case usernamePassword(username: String, password: String)
    }

    /// Socks v5 proxy configuration.
    struct SocksConfiguration: Codable {
        /// Proxy server address.
        var server: AnyIPAddress

        /// Proxy server port.
        var port: UInt16

        /// Authentication method.
        var authentication: SocksAuthentication
    }

    /// Shadowsocks configuration.
    struct ShadowsocksConfiguration: Codable {
        /// Server address.
        var server: AnyIPAddress

        /// Server port.
        var port: UInt16

        /// Server password.
        var password: String

        /// Server cipher.
        var cipher: ShadowsocksCipher
    }
}

extension PersistentAccessMethod {
    /// A kind of access method.
    var kind: AccessMethodKind {
        switch proxyConfiguration {
        case .direct:
            .direct
        case .bridges:
            .bridges
        case .shadowsocks:
            .shadowsocks
        case .socks5:
            .socks5
        }
    }
}

/// A kind of API access method.
enum AccessMethodKind: Equatable, Hashable, CaseIterable {
    /// Direct communication.
    case direct

    /// Communication over bridges.
    case bridges

    /// Communication over shadowsocks.
    case shadowsocks

    /// Communication over socks v5 proxy.
    case socks5
}

extension AccessMethodKind {
    /// Returns `true` if the method is permanent and cannot be deleted.
    var isPermanent: Bool {
        switch self {
        case .direct, .bridges:
            true
        case .shadowsocks, .socks5:
            false
        }
    }

    /// Returns all access method kinds that can be added by user.
    static var allUserDefinedKinds: [AccessMethodKind] {
        allCases.filter { !$0.isPermanent }
    }
}
