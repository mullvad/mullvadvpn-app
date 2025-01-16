//
//  RESTProxyFactory.swift
//  MullvadREST
//
//  Created by pronebird on 19/04/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadRustRuntime

public protocol ProxyFactoryProtocol {
    var configuration: REST.AuthProxyConfiguration { get }

    func createAPIProxy() -> APIQuerying
    func createAccountsProxy() -> RESTAccountHandling
    func createDevicesProxy() -> DeviceHandling

    static func makeProxyFactory(
        transportProvider: RESTTransportProvider,
        addressCache: REST.AddressCache,
        apiContext: MullvadApiContext
    ) -> ProxyFactoryProtocol
}

extension REST {
    public final class ProxyFactory: ProxyFactoryProtocol {
        public var configuration: AuthProxyConfiguration

        public static func makeProxyFactory(
            transportProvider: any RESTTransportProvider,
            addressCache: REST.AddressCache,
            apiContext: MullvadApiContext
        ) -> any ProxyFactoryProtocol {
            let basicConfiguration = REST.ProxyConfiguration(
                transportProvider: transportProvider,
                addressCacheStore: addressCache,
                apiContext: apiContext
            )

            let authenticationProxy = REST.AuthenticationProxy(
                configuration: basicConfiguration
            )
            let accessTokenManager = REST.AccessTokenManager(
                authenticationProxy: authenticationProxy
            )

            let authConfiguration = REST.AuthProxyConfiguration(
                proxyConfiguration: basicConfiguration,
                accessTokenManager: accessTokenManager,
                apiContext: apiContext
            )

            return ProxyFactory(configuration: authConfiguration)
        }

        public init(configuration: AuthProxyConfiguration) {
            self.configuration = configuration
        }

        public func createAPIProxy() -> APIQuerying {
            REST.APIProxy(configuration: configuration)
        }

        public func createAccountsProxy() -> RESTAccountHandling {
            REST.AccountsProxy(configuration: configuration)
        }

        public func createDevicesProxy() -> DeviceHandling {
            REST.DevicesProxy(configuration: configuration)
        }
    }
}
