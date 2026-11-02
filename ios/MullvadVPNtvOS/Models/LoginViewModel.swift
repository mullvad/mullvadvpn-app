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
    func loginWith(_: String) async
    func createAccount() async
}

@Observable
class LoginViewModel: LoggingIn {
    var accountProxy: RESTAccountHandling!
    var deviceProxy: DeviceHandling!
    var state: LoggingInState = .`initial`

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
                // Holds the WireGuard private key and the interface addresses assigned to the device,
                // which is what `SettingsReader` in the packet tunnel reads to build its configuration.
                try settingsManager.writeDeviceState(.loggedIn(storedAccountData, storedDeviceData))
                // The packet tunnel refuses to start without tunnel settings, and nothing else on
                // tvOS seeds them the way `AppDelegate` does on iOS.
                if (try? settingsManager.readSettings()) == nil {
                    try settingsManager.writeSettings(LatestTunnelSettings())
                }
                try settingsManager.setLastUsedAccount(accountNumber)
                // The relay the tunnel connects to is picked from the relay cache, so a logged in
                // device alone is not enough to connect. A failure here must not void the login:
                // the device already exists server-side, and logging in again would burn another
                // device slot. The reload button is the way back from it.
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

/// Fetches the relay list and stores it in the relay cache shared with the packet tunnel.
///
/// The tunnel's relay selector only ever reads this cache, and the prebundled relay list is empty
/// in non-release builds. On iOS `RelayCacheTracker` keeps the cache fresh, but it lives in the app
/// target and depends on UIKit background tasks, so tvOS populates the cache here instead.
struct RelayListUpdater {
    private let logger = Logger(label: "RelayListUpdater")
    private let apiProxy: APIQuerying
    private let relayCache: RelayCacheProtocol

    init(apiProxy: APIQuerying, relayCache: RelayCacheProtocol) {
        self.apiProxy = apiProxy
        self.relayCache = relayCache
    }

    /// Number of WireGuard relays in the cache, and zero when it is empty or unreadable. The
    /// tunnel can only reach a blocked state while this is zero.
    var cachedRelayCount: Int {
        (try? relayCache.read())?.relays.wireguard.relays.count ?? 0
    }

    /// Fetches the relay list and writes it to the cache.
    ///
    /// - Throws: whatever the API or the cache write failed with, so that callers can show it. A
    ///   silent failure here leaves the tunnel with nothing to connect to.
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

        // An unchanged sigsum digest yields an empty body.
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

@Observable
final class LoginViewModelStub: LoggingIn {
    var state: LoggingInState = .`initial`

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
