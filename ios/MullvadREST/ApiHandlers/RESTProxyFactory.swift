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
    var apiTransportProvider: APITransportProviderProtocol { get }

    func createAPIProxy() -> APIQuerying
    func createAccountsProxy() -> RESTAccountHandling
    func createDevicesProxy() -> DeviceHandling

    static func makeProxyFactory(
        apiTransportProvider: APITransportProviderProtocol
    ) -> ProxyFactoryProtocol
}

extension REST {
    public final class ProxyFactory: ProxyFactoryProtocol {
        public let apiTransportProvider: APITransportProviderProtocol

        public static func makeProxyFactory(
            apiTransportProvider: any APITransportProviderProtocol
        ) -> any ProxyFactoryProtocol {
            ProxyFactory(apiTransportProvider: apiTransportProvider)
        }

        public init(apiTransportProvider: APITransportProviderProtocol) {
            self.apiTransportProvider = apiTransportProvider
        }

        public func createAPIProxy() -> APIQuerying {
            MullvadAPIProxy(
                transportProvider: apiTransportProvider,
                dispatchQueue: DispatchQueue(label: "MullvadAPIProxy.dispatchQueue"),
                responseDecoder: Coding.makeJSONDecoder()
            )
        }

        public func createAccountsProxy() -> RESTAccountHandling {
            MullvadAccountProxy(
                transportProvider: apiTransportProvider,
                dispatchQueue: DispatchQueue(label: "MullvadAccountProxy.dispatchQueue"),
                responseDecoder: Coding.makeJSONDecoder()
            )
        }

        public func createDevicesProxy() -> DeviceHandling {
            MullvadDeviceProxy(
                transportProvider: apiTransportProvider,
                dispatchQueue: DispatchQueue(label: "MullvadDeviceProxy.dispatchQueue"),
                responseDecoder: Coding.makeJSONDecoder()
            )
        }
    }
}
