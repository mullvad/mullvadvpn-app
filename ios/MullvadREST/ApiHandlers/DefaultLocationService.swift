// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import CoreLocation
import MullvadLogging
import MullvadTypes

public struct DefaultLocationService {
    private let urlSession: URLSessionProtocol
    private let relayCache: CachedRelays
    private let logger = Logger(label: "DefaultLocationService")

    public init(urlSession: URLSessionProtocol, relayCache: CachedRelays) {
        self.urlSession = urlSession
        self.relayCache = relayCache
    }

    public func fetchCurrentLocationIdentifier() async throws -> REST.LocationIdentifier? {
        // Safe to unwrap since it's a constant.
        let url = URL(string: REST.amIMullvadHostname).unsafelyUnwrapped

        let serverLocation: REST.ServerLocation
        do {
            let data = try await urlSession.data(
                for: URLRequest(url: url, timeoutInterval: REST.defaultAPINetworkTimeout.timeInterval))
            serverLocation = try JSONDecoder().decode(REST.ServerLocation.self, from: data.0)
        } catch {
            logger.log(level: .error, "Could not fetch server location: \(error.description)")
            return nil
        }

        let mappedRelays = RelayWithLocation.locateRelays(
            relays: relayCache.relays.wireguard.relays,
            locations: relayCache.relays.locations
        )

        let closestRelays = RelaySelector.closestRelays(
            to: CLLocationCoordinate2D(latitude: serverLocation.latitude, longitude: serverLocation.longitude),
            using: mappedRelays
        )

        return closestRelays.first?.relay.location
    }
}
