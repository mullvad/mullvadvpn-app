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
    public var apiTransportProvider: APITransportProviderProtocol

    public func createAPIProxy() -> any APIQuerying {
        REST.MullvadAPIProxy(
            transportProvider: apiTransportProvider,
            dispatchQueue: DispatchQueue(label: "MullvadAPIProxy.dispatchQueue"),
            responseDecoder: REST.Coding.makeJSONDecoder()
        )
    }

    public func createAccountsProxy() -> any RESTAccountHandling {
        AccountsProxyStub(createAccountResult: .success(.mockValue()))
    }

    public func createDevicesProxy() -> any DeviceHandling {
        DevicesProxyStub(deviceResult: .success(Device.mock(publicKey: PrivateKey().publicKey)))
    }

    public static func makeProxyFactory(
        apiTransportProvider: any APITransportProviderProtocol
    ) -> any ProxyFactoryProtocol {
        MockProxyFactory(            
            apiTransportProvider: apiTransportProvider
        )
    }
}
