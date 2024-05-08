//
//  AccountsProxy+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

struct AccountsProxyStub: RESTAccountHandling {
    var createAccountResult: Result<REST.NewAccountData, Error>?
    func createAccount(
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<REST.NewAccountData>
    ) -> Cancellable {
        if let createAccountResult = createAccountResult {
            completion(createAccountResult)
        }
        return AnyCancellable()
    }

    func getAccountData(accountNumber: String) -> any RESTRequestExecutor<Account> {
        RESTRequestExecutorStub<Account>(success: {
            Account(
                id: accountNumber,
                expiry: Calendar.current.date(byAdding: .day, value: 38, to: Date())!,
                maxDevices: 1,
                canAddDevices: true
            )
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
