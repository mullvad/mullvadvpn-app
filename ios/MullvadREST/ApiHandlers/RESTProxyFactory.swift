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
