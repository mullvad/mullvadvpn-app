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

enum LoggingInState: Sendable {
    case `default`
    case authenticating
    case success
    case failure(Error)
}

protocol LoggingIn {
    var state: LoggingInState { get set }
    func loginWithNumber(_: String) async
    func createAccount() async
}

@Observable
class LoginViewModel: LoggingIn {
    var accountProxy: RESTAccountHandling!
    var state: LoggingInState = .default

    init(accountProxy: RESTAccountHandling) {
        self.accountProxy = accountProxy
    }

    func loginWithNumber(_ accountNumber: String) async {
        state = .authenticating

        switch await accountProxy.getAccountData(accountNumber: accountNumber, retryStrategy: .apiRequest) {
        case .success:
            state = .success

        case let .failure(error):
            state = .failure(error)
        }
    }

    func createAccount() async {
        state = .authenticating
    }
}

@Observable
final class LoginViewModelStub: LoggingIn {
    var state: LoggingInState = .default

    func loginWithNumber(_: String) async {
        state = .authenticating
        try? await Task.sleep(for: .milliseconds(200))
        state = .success
    }
    
    func createAccount() async {
        state = .authenticating
        try? await Task.sleep(for: .milliseconds(200))
        state = .failure(POSIXError(.ENOSR))
    }
}
