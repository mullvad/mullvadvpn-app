//
//  AccountsProxy+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadREST
@testable import MullvadTypes

struct AccountsProxyStub: RESTAccountHandling {
    func createAccount(
        retryStrategy: REST.RetryStrategy,
        completion: @escaping MullvadREST.ProxyCompletionHandler<REST.NewAccountData>
    ) -> Cancellable {
        AnyCancellable()
    }

    func getAccountData(accountNumber: String) -> any RESTRequestExecutor<Account> {
        RESTRequestExecutorStub<Account>(success: {
            Account(id: accountNumber, expiry: .distantFuture, maxDevices: 1, canAddDevices: true)
        })
    }

    func deleteAccount(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Void>
    ) -> Cancellable {
        AnyCancellable()
    }
}
