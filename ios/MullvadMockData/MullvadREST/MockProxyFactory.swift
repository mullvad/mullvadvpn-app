//
//  MockProxyFactory.swift
//  MullvadMockData
//
//  Created by Mojgan on 2024-05-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadRustRuntime
import MullvadTypes
import WireGuardKitTypes

public struct MockProxyFactory: ProxyFactoryProtocol {
    public var configuration: REST.AuthProxyConfiguration

    public func createAPIProxy() -> any APIQuerying {
        REST.APIProxy(configuration: configuration)
    }

    public func createAccountsProxy() -> any RESTAccountHandling {
        AccountsProxyStub(createAccountResult: .success(.mockValue()))
    }

    public func createDevicesProxy() -> any DeviceHandling {
        DevicesProxyStub(deviceResult: .success(Device.mock(publicKey: PrivateKey().publicKey)))
    }

    public static func makeProxyFactory(
        transportProvider: any RESTTransportProvider,
        apiTransportProvider: any APITransportProviderProtocol,
        addressCache: REST.AddressCache
    ) -> any ProxyFactoryProtocol {
        let basicConfiguration = REST.ProxyConfiguration(
            transportProvider: transportProvider,
            apiTransportProvider: apiTransportProvider,
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

        return MockProxyFactory(configuration: authConfiguration)
    }
}
