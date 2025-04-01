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
    var createAccountResult: Result<NewAccountData, Error> = .failure(AccountProxyStubError())
    var deleteAccountResult: Result<Void, Error> = .failure(AccountProxyStubError())
    func createAccount(
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<NewAccountData>
    ) -> Cancellable {
        completion(createAccountResult)
        return AnyCancellable()
    }

    func getAccountData(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy,
        completion: @escaping ProxyCompletionHandler<Account>
    ) -> Cancellable {
        completion(.success(Account(
            id: accountNumber,
            expiry: Calendar.current.date(byAdding: .day, value: 38, to: Date())!,
            maxDevices: 1,
            canAddDevices: true
        )))
        return AnyCancellable()
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
