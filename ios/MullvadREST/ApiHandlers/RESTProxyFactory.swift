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
import MullvadRustRuntime

public protocol ProxyFactoryProtocol {
    var apiTransportProvider: APITransportProviderProtocol { get }

    func createAPIProxy() -> APIQuerying
    func createAccountsProxy() -> RESTAccountHandling
    func createDevicesProxy() -> DeviceHandling

    static func makeProxyFactory(
        apiContext: ApiContext,
        apiTransportProvider: APITransportProviderProtocol
    ) -> ProxyFactoryProtocol
}

extension REST {
    public final class ProxyFactory: ProxyFactoryProtocol {
        public let apiTransportProvider: APITransportProviderProtocol
        public let apiContext: ApiContext

        public static func makeProxyFactory(
            apiContext: ApiContext,
            apiTransportProvider: any APITransportProviderProtocol
        ) -> any ProxyFactoryProtocol {
            ProxyFactory(apiContext: apiContext, apiTransportProvider: apiTransportProvider)
        }

        public init(apiContext: ApiContext, apiTransportProvider: APITransportProviderProtocol) {
            self.apiContext = apiContext
            self.apiTransportProvider = apiTransportProvider
        }

        public func createAPIProxy() -> APIQuerying {
            MullvadAPIProxy(
                apiContext: apiContext,
                transportProvider: apiTransportProvider,
                dispatchQueue: DispatchQueue(label: "MullvadAPIProxy.dispatchQueue"),
                responseDecoder: Coding.makeJSONDecoder(),
                encoder: REST.Coding.makeJSONEncoder()
            )
        }

        public func createAccountsProxy() -> RESTAccountHandling {
            MullvadAccountProxy(
                apiContext: apiContext,
                transportProvider: apiTransportProvider,
                dispatchQueue: DispatchQueue(label: "MullvadAccountProxy.dispatchQueue"),
                responseDecoder: Coding.makeJSONDecoder()
            )
        }

        public func createDevicesProxy() -> DeviceHandling {
            MullvadDeviceProxy(
                apiContext: apiContext,
                transportProvider: apiTransportProvider,
                dispatchQueue: DispatchQueue(label: "MullvadDeviceProxy.dispatchQueue"),
                responseDecoder: Coding.makeJSONDecoder()
            )
        }
    }
}
