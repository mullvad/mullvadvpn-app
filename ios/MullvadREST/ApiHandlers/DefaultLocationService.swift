//
//  DefaultLocationService.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
            logger.log(level: .error, "Could not fetch server location: \(error.localizedDescription)")
            return nil
        }

        let mappedRelays = RelayWithLocation.locateRelays(
            relays: relayCache.relays.wireguard.relays,
            locations: relayCache.relays.locations
        )

        let closestRelay = RelaySelector.WireGuard.closestRelay(
            to: CLLocationCoordinate2D(latitude: serverLocation.latitude, longitude: serverLocation.longitude),
            using: mappedRelays
        )

        return closestRelay?.location
    }
}
