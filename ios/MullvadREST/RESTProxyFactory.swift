//
//  RESTProxyFactory.swift
//  MullvadREST
//
//  Created by pronebird on 19/04/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

extension REST {
    public final class ProxyFactory {
        public let configuration: AuthProxyConfiguration

        public init(
            transportRegistry: TransportRegistry,
            addressCacheStore: AddressCache
        ) {
            let basicConfiguration = ProxyConfiguration(
                transportRegistry: transportRegistry,
                addressCacheStore: addressCacheStore
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

            configuration = authConfiguration
        }

        public init(configuration: AuthProxyConfiguration) {
            self.configuration = configuration
        }

        public convenience init(
            transportRegistry: TransportRegistry = .shared,
            addressCacheStoreAccessLevel: AccessPermission
        ) {
            self.init(transportRegistry: transportRegistry,
                      addressCacheStore: AddressCache(accessLevel: addressCacheStoreAccessLevel))
        }

        public func createAPIProxy() -> REST.APIProxy {
            return REST.APIProxy(configuration: configuration)
        }

        public func createAccountsProxy() -> REST.AccountsProxy {
            return REST.AccountsProxy(configuration: configuration)
        }

        public func createDevicesProxy() -> REST.DevicesProxy {
            return REST.DevicesProxy(configuration: configuration)
        }
    }
}
