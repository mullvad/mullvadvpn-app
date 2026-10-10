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
import MullvadLogging
import MullvadREST
import MullvadTypes
import MullvadRustRuntime
import MullvadSettings

enum LoggingInState: Sendable, Equatable {
    static func == (lhs: LoggingInState, rhs: LoggingInState) -> Bool {
        switch (lhs, rhs) {
        case (.initial, .initial): true
        case (.authenticating, .authenticating): true
        case (.success, .success): true
        case (let .failure(lhsError), let .failure(rhsError)): lhsError.description == rhsError.description
        default: false
        }
    }

    case initial
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
    var deviceProxy: DeviceHandling!
    var state: LoggingInState = .`initial`

    init(accountProxy: RESTAccountHandling, deviceProxy: DeviceHandling) {
        self.accountProxy = accountProxy
        self.deviceProxy = deviceProxy
    }

    func loginWithNumber(_ accountNumber: String) async {
        state = .authenticating

        // FIXME: Harmonize the flow to login, all the APIs used should look the same (either with a Result type or throwing flow)
        switch await accountProxy.getAccountData(accountNumber: accountNumber, retryStrategy: .apiRequest) {
        case let .success(account):
            let privateKey = WireGuard.PrivateKey()
            let request = CreateDeviceRequest(publicKey: privateKey.publicKey, hijackDNS: false)
            do {
                // FIXME: Handle retry strategy errors and delays when the public key we just created is already in use
                let device = try await deviceProxy.createDevice(accountNumber: accountNumber, request: request, retryStrategy: .apiRequest)
                // FIXME: Should this be responsible for storing the device data, or should another service do that ?
                let storedAccountData = StoredAccountData(identifier: account.id, number: accountNumber, expiry: account.expiry)
                let storedDeviceData = StoredDeviceData(with: device, privateKey: privateKey)
                let loggedInState = DeviceState.loggedIn(storedAccountData, storedDeviceData)
                let data = try JSONEncoder().encode(loggedInState)
                try SettingsManager().store.write(data, for: .deviceState)
                state = .success
            } catch {
                state = .failure(error)
            }

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
    var state: LoggingInState = .`initial`

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
