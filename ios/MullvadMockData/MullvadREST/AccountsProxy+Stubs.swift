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
import MullvadTypes

struct AccountProxyStubError: Error {}

struct AccountsProxyStub: RESTAccountHandling {
    var createAccountResult: Result<NewAccountData, Error> = .failure(AccountProxyStubError())
    var deleteAccountResult: Result<Void, Error> = .failure(AccountProxyStubError())

    func createAccount(
        retryStrategy: REST.RetryStrategy
    ) async -> Result<NewAccountData, Error> {
        createAccountResult
    }

    func getAccountData(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Account, Error> {
        .success(
            Account(
                id: accountNumber,
                expiry: Calendar.current.date(byAdding: .day, value: 38, to: Date())!,
                maxDevices: 1,
                canAddDevices: true
            )
        )
    }

    func deleteAccount(
        accountNumber: String,
        retryStrategy: REST.RetryStrategy
    ) async -> Result<Void, Error> {
        deleteAccountResult
    }
}
