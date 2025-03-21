//
//  AccountsProxy+Stubs.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-03.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes

struct AccountProxyStubError: Error {}

struct AccountsProxyStub: RESTAccountHandling {
    var createAccountResult: Result<REST.NewAccountData, Error> = .failure(AccountProxyStubError())
    var deleteAccountResult: Result<Void, Error> = .failure(AccountProxyStubError())
    func createAccount(
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<REST.NewAccountData>
    ) -> Cancellable {
        completion(createAccountResult)
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
        completion(deleteAccountResult)
        return AnyCancellable()
    }
}
