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
import MullvadRustRuntime
import MullvadTypes

public struct DefaultLocationService {
    private let relayCache: CachedRelays
    private let apiContext: ApiContext
    private let logger = Logger(label: "DefaultLocationService")
    private let endpoint: String

    public init(relayCache: CachedRelays, apiContext: ApiContext, endpoint: String = REST.amIMullvadHostname) {
        self.relayCache = relayCache
        self.apiContext = apiContext
        self.endpoint = endpoint
    }

    public func getLocation(_ response: AmIMullvadResponse) -> REST.LocationIdentifier? {
        let mappedRelays = RelayWithLocation.locateRelays(
            relays: relayCache.relays.wireguard.relays,
            locations: relayCache.relays.locations
        )

        let closestRelays = RelaySelector.closestRelays(
            to: CLLocationCoordinate2D(latitude: response.latitude, longitude: response.longitude),
            using: mappedRelays
        )

        return closestRelays.first?.relay.location
    }

    public func fetchCurrentLocationIdentifier() async throws -> REST.LocationIdentifier? {
        guard
            let serverLocation = await apiContext.amIMullvad(
                address: "https://\(endpoint)/json",
                retryStrategy: REST.RetryStrategy.noRetry.toRustStrategy())
        else {
            return nil
        }

        return getLocation(serverLocation)
    }
}
