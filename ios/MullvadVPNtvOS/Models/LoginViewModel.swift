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
import MullvadRustRuntime
import MullvadSettings
import MullvadTypes

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
    var isLoggedIn: Bool { get }
    func loginWith(_: String) async
    func createAccount() async
}

@Observable
class LoginViewModel: LoggingIn {
    var accountProxy: RESTAccountHandling!
    var deviceProxy: DeviceHandling!
    var state: LoggingInState = .`initial`

    /// Whether settings hold a device, either from a login just now or from an earlier launch.
    var isLoggedIn: Bool {
        do {
            return try settingsManager.readDeviceState().isLoggedIn
        } catch {
            // Expected when no one has logged in on this install yet, and the only trace of a
            // keychain that refused the read otherwise.
            logger.debug("Could not read a device state: \(error)")
            return false
        }
    }

    private let logger = Logger(label: "LoginViewModel")
    private let settingsManager: SettingsManager
    private let relayListUpdater: RelayListUpdater

    init(
        accountProxy: RESTAccountHandling,
        deviceProxy: DeviceHandling,
        settingsManager: SettingsManager,
        relayListUpdater: RelayListUpdater
    ) {
        self.accountProxy = accountProxy
        self.deviceProxy = deviceProxy
        self.settingsManager = settingsManager
        self.relayListUpdater = relayListUpdater
    }

    func loginWith(_ accountNumber: String) async {
        state = .authenticating

        // FIXME: Harmonize the flow to login, all the APIs used should look the same (either with a Result type or throwing flow)
        switch await accountProxy.getAccountData(accountNumber: accountNumber, retryStrategy: .apiRequest) {
        case let .success(account):
            let privateKey = WireGuard.PrivateKey()
            let request = CreateDeviceRequest(publicKey: privateKey.publicKey, hijackDNS: false)
            do {
                // FIXME: Handle retry strategy errors and delays when the public key we just created is already in use
                let device = try await deviceProxy.createDevice(
                    accountNumber: accountNumber, request: request, retryStrategy: .apiRequest)
                // FIXME: Should this be responsible for storing the device data, or should another service do that ?
                let storedAccountData = StoredAccountData(
                    identifier: account.id, number: accountNumber, expiry: account.expiry)
                let storedDeviceData = StoredDeviceData(with: device, privateKey: privateKey)

                try settingsManager.writeDeviceState(.loggedIn(storedAccountData, storedDeviceData))

                // Initialize settings here if they are empty.
                if (try? settingsManager.readSettings()) == nil {
                    try settingsManager.writeSettings(LatestTunnelSettings())
                }
                try settingsManager.setLastUsedAccount(accountNumber)
                // Updating the relay list because debug builds do not bundle one
                do {
                    try await relayListUpdater.updateRelays()
                } catch {
                    logger.error(error: error, message: "Failed to update relay list on login.")
                }
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

/// Updates relay list when logging in
protocol RelayListUpdating: Sendable {
    /// Number of WireGuard relays in the cache, and zero when it is empty or unreadable.
    var cachedRelayCount: Int { get }
    func updateRelays() async throws
}

/// Fetches the relay list and stores it in the relay cache shared with the packet tunnel.
///
/// The tunnel's relay selector only ever reads this cache, and the prebundled relay list is empty
/// in non-release builds. On iOS `RelayCacheTracker` keeps the cache fresh, but it lives in the app
/// target and depends on UIKit background tasks, so tvOS populates the cache here instead.
struct RelayListUpdater: RelayListUpdating {
    private let logger = Logger(label: "RelayListUpdater")
    private let apiProxy: APIQuerying
    private let relayCache: RelayCacheProtocol

    init(apiProxy: APIQuerying, relayCache: RelayCacheProtocol) {
        self.apiProxy = apiProxy
        self.relayCache = relayCache
    }

    /// Number of WireGuard relays in the cache, and zero when it is empty or unreadable.
    var cachedRelayCount: Int {
        (try? relayCache.read())?.relays.wireguard.relays.count ?? 0
    }

    /// Fetches the relay list and writes it to the cache.
    func updateRelays() async throws {
        let cachedRelays = try? relayCache.read()
        var sigsum: (String, Int64)?
        if let digest = cachedRelays?.digest, let timestamp = cachedRelays?.timestamp {
            sigsum = (digest, timestamp)
        }

        let response: REST.ServerRelaysCacheResponse? = try await withCheckedThrowingContinuation {
            continuation in
            _ = apiProxy.getRelays(sigsum: sigsum, retryStrategy: .apiRequest) { result in
                continuation.resume(with: result)
            }
        }

        switch response ?? .notModified {
        case let .newContent(digest, timestamp, rawData):
            try relayCache.write(
                record: StoredRelays(
                    digest: digest, timestamp: timestamp, rawData: rawData, updatedAt: Date()))
            logger.debug("Updated cached relay list.")
        case .notModified:
            logger.debug("Cached relay list is up to date.")
        }
    }
}

struct RelayListUpdaterStub: RelayListUpdating {
    var cachedRelayCount = 0

    func updateRelays() async throws {}
}

@Observable
final class LoginViewModelStub: LoggingIn {
    var state: LoggingInState = .`initial`
    var isLoggedIn = false

    func loginWith(_: String) async {
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
