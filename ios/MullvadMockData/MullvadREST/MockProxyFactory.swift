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
import MullvadTypes

public struct MockProxyFactory: ProxyFactoryProtocol {
    public var apiTransportProvider: APITransportProviderProtocol

    public func createAPIProxy() -> any APIQuerying {
        APIProxyStub()
    }

    public func createAccountsProxy() -> any RESTAccountHandling {
        AccountsProxyStub(createAccountResult: .success(.mockValue()))
    }

    public func createDevicesProxy() -> any DeviceHandling {
        DevicesProxyStub(deviceResult: .success(Device.mock(publicKey: WireGuard.PrivateKey().publicKey)))
    }

    public static func makeProxyFactory(
        apiTransportProvider: any APITransportProviderProtocol
    ) -> any ProxyFactoryProtocol {
        MockProxyFactory(
            apiTransportProvider: apiTransportProvider
        )
    }
}
