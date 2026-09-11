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

    public init(relayCache: CachedRelays, apiContext: ApiContext) {
        self.relayCache = relayCache
        self.apiContext = apiContext
    }

    public func fetchCurrentLocationIdentifier() async throws -> REST.LocationIdentifier? {
        let serverLocation: IAmMullvadResponse
        do {
            serverLocation = try await apiContext.amIMullvad(
                useIpv6: false,
                hostname: REST.amIMullvadHostname,
                retryStrategy: REST.RetryStrategy.noRetry.toRustStrategy())
        } catch let error as ErasedError {
            logger.error("failed to fetch location: \(error.asString())")
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
