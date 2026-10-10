// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadLogging
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes

@MainActor
@Observable
class DependencyManager {
    // TODO: Cut this into proper dependencies, it shouldn't be part of 2 different frameworks
    private let settingsManager: SettingsManager
    private let shadowsocksLoader: ShadowsocksLoader
    private let accessMethodReceiver: MullvadAccessMethodReceiver
    private let accessMethodRepository: AccessMethodRepository
    private let apiContext: ApiContext
    private let shadowsocksCacheCleaner: ShadowsocksCacheCleaner
    private let ipOverrideRepository: IPOverrideRepository
    private let proxyFactory: ProxyFactoryProtocol

    private init(
        settingsManager: SettingsManager,
        shadowsocksLoader: ShadowsocksLoader,
        accessMethodReceiver: MullvadAccessMethodReceiver,
        accessMethodRepository: AccessMethodRepository,
        apiContext: ApiContext,
        shadowsocksCacheCleaner: ShadowsocksCacheCleaner,
        ipOverrideRepository: IPOverrideRepository,
        proxyFactory: ProxyFactoryProtocol
    ) {
        self.settingsManager = settingsManager
        self.shadowsocksLoader = shadowsocksLoader
        self.accessMethodReceiver = accessMethodReceiver
        self.accessMethodRepository = accessMethodRepository
        self.apiContext = apiContext
        self.shadowsocksCacheCleaner = shadowsocksCacheCleaner
        self.ipOverrideRepository = ipOverrideRepository
        self.proxyFactory = proxyFactory
    }

    static func live() async -> DependencyManager {
        let settingsManager = SettingsManager()
        let containerURL = MullvadSettings.ApplicationConfiguration.containerURL
        let ipOverrideRepository = IPOverrideRepository(
            settingsStore: settingsManager.store
        )
        let ipOverrideWrapper = IPOverrideWrapper(
            relayCache: RelayCache(cacheDirectory: containerURL),
            ipOverrideRepository: ipOverrideRepository
        )

        let tunnelSettingsListener = TunnelSettingsListener()
        let tunnelSettingsUpdater = SettingsUpdater(listener: tunnelSettingsListener)

        let shadowsocksCache = ShadowsocksConfigurationCache(cacheDirectory: containerURL)
        let shadowsocksRelaySelector = ShadowsocksRelaySelector(
            relayCache: ipOverrideWrapper
        )

        let tunnelSettings = (try? settingsManager.readSettings()) ?? LatestTunnelSettings()

        let shadowsocksLoader = ShadowsocksLoader(
            cache: shadowsocksCache,
            relaySelector: shadowsocksRelaySelector,
            tunnelSettings: tunnelSettings,
            settingsUpdater: tunnelSettingsUpdater
        )

        let shadowsocksCacheCleaner = ShadowsocksCacheCleaner(cache: shadowsocksCache)

        let accessMethodRepository = AccessMethodRepository(
            shadowsocksCiphers: ShadowsocksCipherService().getCiphers(),
            settingsStore: settingsManager.store
        )
        let opaqueAccessMethodSettingsWrapper = initAccessMethodSettingsWrapper(
            methods: accessMethodRepository.fetchAll()
        )

        let apiContext = ApiContext(
            host: REST.defaultAPIHostname,
            address: REST.defaultAPIEndpoint.description,
            amIMullvadHostIpv4: "ipv4.\(REST.amIMullvadHostname)",
            amIMullvadHostIpv6: "ipv6.\(REST.amIMullvadHostname)",
            domain: REST.encryptedDNSHostname,
            domainFronting: REST.domainFronting,
            sigsumTrustedKeys: REST.sigsumTrustedKeys,
            shadowsocksProvider: shadowsocksLoader,
            accessMethodWrapper: opaqueAccessMethodSettingsWrapper,
            accessMethodChangeListeners: [accessMethodRepository, shadowsocksCacheCleaner]
        )

        let accessMethodReceiver = MullvadAccessMethodReceiver(
            apiContext: apiContext,
            validShadowsocksCiphers: accessMethodRepository.shadowsocksCiphers,
            accessMethodsDataSource: accessMethodRepository.accessMethodsPublisher,
            requestDataSource: accessMethodRepository.requestAccessMethodPublisher
        )

        let apiRequestFactory = MullvadApiRequestFactory(
            apiContext: apiContext,
            encoder: REST.Coding.makeJSONEncoder()
        )
        let apiTransportProvider = APITransportProvider(requestFactory: apiRequestFactory)
        let proxyFactory = REST.ProxyFactory.makeProxyFactory(apiTransportProvider: apiTransportProvider)

        return DependencyManager(
            settingsManager: settingsManager,
            shadowsocksLoader: shadowsocksLoader,
            accessMethodReceiver: accessMethodReceiver,
            accessMethodRepository: accessMethodRepository,
            apiContext: apiContext,
            shadowsocksCacheCleaner: shadowsocksCacheCleaner,
            ipOverrideRepository: ipOverrideRepository,
            proxyFactory: proxyFactory
        )
    }

    public func makeLoginViewModel() -> any LoggingIn {
        LoginViewModel(accountProxy: proxyFactory.createAccountsProxy(), deviceProxy: proxyFactory.createDevicesProxy())
    }
}
