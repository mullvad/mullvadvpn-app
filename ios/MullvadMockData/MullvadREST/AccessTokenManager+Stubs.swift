//
//  AccessTokenManager+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

struct AccessTokenManagerStub: RESTAccessTokenManagement {
    func getAccessToken(
        accountNumber: String,
        completionHandler: @escaping ProxyCompletionHandler<REST.AccessTokenData>
    ) -> Cancellable {
        AnyCancellable()
    }

    func invalidateAllTokens() {}
}
