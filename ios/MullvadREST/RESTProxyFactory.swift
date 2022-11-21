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

        public class func makeProxyFactory(
            transportProvider: @escaping () -> RESTTransport?,
            addressCache: AddressCache
        ) -> ProxyFactory {
            let basicConfiguration = REST.ProxyConfiguration(
                transportProvider: transportProvider,
                addressCacheStore: addressCache
            )

            let authenticationProxy = REST.AuthenticationProxy(
                configuration: basicConfiguration
            )
            let accessTokenManager = REST.AccessTokenManager(
                authenticationProxy: authenticationProxy
            )

            let authConfiguration = REST.AuthProxyConfiguration(
                proxyConfiguration: basicConfiguration,
                accessTokenManager: accessTokenManager
            )

            return ProxyFactory(configuration: authConfiguration)
        }

        public init(configuration: AuthProxyConfiguration) {
            self.configuration = configuration
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
