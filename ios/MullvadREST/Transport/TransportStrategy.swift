//
//  TransportStrategy.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2023-04-27.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging
import MullvadSettings
import MullvadTypes

public class TransportStrategy: Equatable {
    /// The different transports suggested by the strategy
    public enum Transport {
        /// Connecting a direct connection
        case direct

        /// Connecting via custom Shadowsocks servers
        case shadowsocks(configuration: ShadowsocksConfiguration)

        /// Connecting via socks proxy
        case socks5(configuration: Socks5Configuration)
    }

    /// `UserDefaults` key shared by both processes. Used to cache and synchronize last reachable api access method between them.
    internal static let lastReachableConfigurationCacheKey = "LastReachableConfigurationCacheKey"

    /// Enables recording of last reachable configuration .
    private var lastReachableConfigurationId: UUID {
        get {
            storage.id
        } set {
            storage.id = newValue
        }
    }

    /// Fetches user's configuration
    private var dataSource: AccessMethodRepositoryDataSource

    private var storage: LastReachableApiAccessStorage

    private var shadowsocksLoader: ShadowsocksLoaderProtocol

    private let logger = Logger(label: "TransportStrategy")

    var onNewConstraints: ((RelayConstraints) -> Void)?

    public init(
        _ userDefaults: UserDefaults,
        datasource: AccessMethodRepositoryDataSource,
        shadowsocksLoader: ShadowsocksLoaderProtocol
    ) {
        self.dataSource = datasource
        self.shadowsocksLoader = shadowsocksLoader
        self.storage = LastReachableApiAccessStorage(
            key: Self.lastReachableConfigurationCacheKey,
            defaultValue: datasource.directAccess.id,
            container: userDefaults
        )
        self.onNewConstraints = { [weak self] values in
            shadowsocksLoader.onUpdatedConstraints?(values)
        }
    }

    /// Rotating between enabled configurations by what order they were added in
    public func didFail() {
        let configuration = nextConfiguration()
        lastReachableConfigurationId = configuration.id
    }

    /// The suggested connection transport
    public func connectionTransport() -> Transport {
        switch currentConfiguration().proxyConfiguration {
        case .direct:
            return .direct
        case .bridges:
            do {
                let configuration = try shadowsocksLoader.configuration
                return .shadowsocks(configuration: configuration)
            } catch {
                logger.error(error: error, message: error.localizedDescription)
                didFail()
                return connectionTransport()
            }
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

    /// Picking the next `Enabled` configuration in order they are added
    /// When reaching the end of the list then it starts from the beginning (direct) again
    /// Returning `Direct` if there is no enabled configuration
    private func nextConfiguration() -> PersistentAccessMethod {
        let direct = dataSource.directAccess
        let enabledConfigurations = dataSource.accessMethods.filter { $0.isEnabled }
        if enabledConfigurations.isEmpty {
            return direct
        } else {
            let totalCount = enabledConfigurations.count
            var currentIndex = enabledConfigurations.firstIndex(where: {
                $0.id == lastReachableConfigurationId
            }) ?? totalCount
            currentIndex += 1
            let next = currentIndex % totalCount
            return enabledConfigurations[next]
        }
    }

    /// Fetching configuration by `lastReachableConfigurationId` and pick the next enabled configuration  if the current cached configuration is disabled
    private func currentConfiguration() -> PersistentAccessMethod {
        if let currentConfiguration = dataSource.accessMethods.first(where: {
            $0.id == lastReachableConfigurationId && $0.isEnabled
        }) {
            return currentConfiguration
        } else {
            let currentConfiguration = nextConfiguration()
            lastReachableConfigurationId = currentConfiguration.id
            return currentConfiguration
        }
    }

    public static func == (lhs: TransportStrategy, rhs: TransportStrategy) -> Bool {
        lhs.lastReachableConfigurationId == rhs.lastReachableConfigurationId
    }
}
