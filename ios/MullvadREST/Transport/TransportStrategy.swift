//
//  TransportStrategy.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

public class TransportStrategy: Equatable {
    /// The different transports suggested by the strategy
    public enum Transport {
        /// Connecting a direct connection
        case direct

        /// Connecting via local Shadowsocks proxy
        case bridge

        /// Connecting via custom Shadowsocks servers
        case shadowsocks(configuration: ShadowsocksConfiguration)

        /// Connecting via socks proxy
        case socks5(configuration: Socks5Configuration)
    }

    private let userDefaults: UserDefaults

    /// `UserDefaults` key shared by both processes. Used to cache and synchronize last reachable api access method between them.
    internal static let lastReachableConfigurationCacheKey = "LastReachableConfigurationCacheKey"

    /// Enables recording of last reachable configuration .
    private var lastReachableConfigurationId: UUID

    /// Fetches user's configuration
    private var dataSource: AccessMethodRepositoryDataSource

    public init(
        _ userDefaults: UserDefaults,
        datasource: AccessMethodRepositoryDataSource
    ) {
        self.userDefaults = userDefaults
        self.dataSource = datasource
        self.lastReachableConfigurationId = UUID(
            uuidString: userDefaults
                .string(forKey: Self.lastReachableConfigurationCacheKey) ?? ""
        ) ?? datasource.accessMethods
            .first(where: { $0.kind == .direct })!.id
    }

    /// Rotating between enabled configurations by what order they were added in
    public func didFail() {
        let configuration = next()
        lastReachableConfigurationId = configuration.id
        userDefaults.set(lastReachableConfigurationId.uuidString, forKey: Self.lastReachableConfigurationCacheKey)
    }

    /// The suggested connection transport
    public func connectionTransport() -> Transport {
        switch configuration.proxyConfiguration {
        case .direct:
            return .direct
        case .bridges:
            return .bridge
        case let .shadowsocks(configuration):
            return .shadowsocks(configuration: ShadowsocksConfiguration(
                address: configuration.server,
                port: configuration.port,
                password: configuration.password,
                cipher: configuration.cipher.rawValue.description
            ))
        case let .socks5(configuration):
            return .socks5(configuration: Socks5Configuration(address: configuration.server, port: configuration.port))
        }
    }

    public static func == (lhs: TransportStrategy, rhs: TransportStrategy) -> Bool {
        lhs.lastReachableConfigurationId == rhs.lastReachableConfigurationId
    }

    /// Picking the next `Enabled` configuration in order they are added
    /// When reaching the end of the list then it starts from the beginning (direct) again
    /// Returning `Direct` if there is no enabled configuration
    private func next() -> PersistentAccessMethod {
        let direct = dataSource.accessMethods.first(where: { $0.kind == .direct })!
        let enabledConfigurations = dataSource.accessMethods.filter { $0.isEnabled }
        if enabledConfigurations.isEmpty {
            return direct
        } else {
            let currentIndex = enabledConfigurations.firstIndex(where: { $0.id == lastReachableConfigurationId }) ?? -1
            let next = currentIndex + 1 >= enabledConfigurations.count ? 0 : currentIndex + 1
            return enabledConfigurations[next]
        }
    }

    /// Fetching configuration by `lastReachableConfigurationId` and pick the next enabled configuration  if the current cached configuration is disabled
    private var configuration: PersistentAccessMethod {
        if let currentConfiguration = dataSource.accessMethods.first(where: {
            $0.id == lastReachableConfigurationId && $0.isEnabled
        }) {
            return currentConfiguration
        } else {
            let currentConfiguration = next()
            lastReachableConfigurationId = currentConfiguration.id
            userDefaults.set(lastReachableConfigurationId.uuidString, forKey: Self.lastReachableConfigurationCacheKey)
            return currentConfiguration
        }
    }
}
