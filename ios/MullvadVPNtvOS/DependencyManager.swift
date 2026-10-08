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
import MullvadREST
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
import MullvadLogging

actor DependencyManager {

    init() {}

    // TODO: Cut this into proper dependencies, it shouldn't be part of 2 different frameworks
    private let containerURL = MullvadSettings.ApplicationConfiguration.containerURL
    private let settingsManager = SettingsManager()
    private(set) var shadowsocksLoader: ShadowsocksLoader!
    var accessMethodReceiver: MullvadAccessMethodReceiver!
    private(set) var accessMethodRepository: AccessMethodRepository!
    private var apiContext: ApiContext!
    private var shadowsocksCacheCleaner: ShadowsocksCacheCleaner!
    private lazy var ipOverrideRepository = IPOverrideRepository(settingsStore: settingsManager.store)
    private var proxyFactory: ProxyFactoryProtocol!

    func start() async -> Self {
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

        shadowsocksLoader = ShadowsocksLoader(
            cache: shadowsocksCache,
            relaySelector: shadowsocksRelaySelector,
            tunnelSettings: tunnelSettings,
            settingsUpdater: tunnelSettingsUpdater
        )

        shadowsocksCacheCleaner = ShadowsocksCacheCleaner(cache: shadowsocksCache)

        accessMethodRepository = AccessMethodRepository(
            shadowsocksCiphers: ShadowsocksCipherService().getCiphers(),
            settingsStore: settingsManager.store
        )
        let opaqueAccessMethodSettingsWrapper = initAccessMethodSettingsWrapper(
            methods: accessMethodRepository.fetchAll()
        )

        apiContext = ApiContext(
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

        accessMethodReceiver = MullvadAccessMethodReceiver(
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
        proxyFactory = REST.ProxyFactory.makeProxyFactory(apiTransportProvider: apiTransportProvider)

        return self
    }
}
