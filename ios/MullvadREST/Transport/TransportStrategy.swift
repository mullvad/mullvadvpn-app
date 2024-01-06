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

        /// Connecting via custom Shadowsocks servers
        case shadowsocks(configuration: ShadowsocksConfiguration?)

        /// Connecting via socks proxy
        case socks5(configuration: Socks5Configuration)
    }

    /// `UserDefaults` key shared by both processes. Used to cache and synchronize last reachable api access method between them.
    internal static let lastReachableConfigurationCacheKey = "LastReachableConfigurationCacheKey"

    /// Enables recording of last reachable configuration .
    private var lastReachableConfigurationId: UUID {
        get {
            wrapper.wrappedValue
        } set {
            wrapper.wrappedValue = newValue
        }
    }

    /// Fetches user's configuration
    private var dataSource: AccessMethodRepositoryDataSource

    private var wrapper: UUIDWrapper

    public init(
        _ userDefaults: UserDefaults,
        datasource: AccessMethodRepositoryDataSource
    ) {
        self.dataSource = datasource
        self.wrapper = UUIDWrapper(
            key: Self.lastReachableConfigurationCacheKey,
            defaultValue: datasource.directAccess.id,
            container: userDefaults
        )
    }

    /// Rotating between enabled configurations by what order they were added in
    public func didFail() {
        let configuration = next()
        lastReachableConfigurationId = configuration.id
    }

    /// The suggested connection transport
    public func connectionTransport() -> Transport {
        switch configuration.proxyConfiguration {
        case .direct:
            return .direct
        case .bridges:
            return .shadowsocks(configuration: nil)
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
    private var configuration: PersistentAccessMethod {
        if let currentConfiguration = dataSource.accessMethods.first(where: {
            $0.id == lastReachableConfigurationId && $0.isEnabled
        }) {
            return currentConfiguration
        } else {
            let currentConfiguration = next()
            lastReachableConfigurationId = currentConfiguration.id
            return currentConfiguration
        }
    }
}

fileprivate extension AccessMethodRepositoryDataSource {
    var directAccess: PersistentAccessMethod {
        accessMethods.first(where: { $0.kind == .direct })!
    }
}

private struct UUIDWrapper {
    private var appStorage: AppStorage<Data>

    private let transformer: Transformer<UUID> = {
        let toData: (UUID) -> Data = { value in
            return value.uuidString.data(using: .ascii)!
        }
        let fromData: (Data) -> UUID = { data in
            let str = String(data: data, encoding: .ascii)
            return UUID(uuidString: str!)!
        }
        return Transformer(toData: toData, fromData: fromData)
    }()

    init(key: String, defaultValue: UUID, container: UserDefaults) {
        self
            .appStorage = AppStorage(
                wrappedValue: transformer.toData(defaultValue),
                key: key,
                container: container
            )
    }

    var wrappedValue: UUID {
        mutating get {
            let value = appStorage.wrappedValue
            return transformer.fromData(value)
        }
        set {
            appStorage.wrappedValue = transformer.toData(newValue)
        }
    }
}

struct Transformer<T> {
    let toData: (T) -> Data
    let fromData: (Data) -> T

    init(toData: @escaping (T) -> Data, fromData: @escaping (Data) -> T) {
        self.toData = toData
        self.fromData = fromData
    }
}
