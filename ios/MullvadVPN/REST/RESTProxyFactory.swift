//
//  RESTProxyFactory.swift
//  MullvadVPN
//
//  Created by pronebird on 19/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    class ProxyFactory {
        let configuration: AuthProxyConfiguration

        static let shared: ProxyFactory = {
            let basicConfiguration = ProxyConfiguration(
                transportRegistry: RESTTransportRegistry.shared,
                addressCacheStore: AddressCache.Store.shared
            )

            let authenticationProxy = REST.AuthenticationProxy(
                configuration: basicConfiguration
            )
            let accessTokenManager = AccessTokenManager(
                authenticationProxy: authenticationProxy
            )

            let authConfiguration = AuthProxyConfiguration(
                proxyConfiguration: basicConfiguration,
                accessTokenManager: accessTokenManager
            )
            return ProxyFactory(configuration: authConfiguration)
        }()

        init(configuration: AuthProxyConfiguration) {
            self.configuration = configuration
        }

        func createAPIProxy() -> REST.APIProxy {
            return REST.APIProxy(configuration: configuration)
        }

        func createAccountsProxy() -> REST.AccountsProxy {
            return REST.AccountsProxy(configuration: configuration)
        }

        func createDevicesProxy() -> REST.DevicesProxy {
            return REST.DevicesProxy(configuration: configuration)
        }
    }
}
