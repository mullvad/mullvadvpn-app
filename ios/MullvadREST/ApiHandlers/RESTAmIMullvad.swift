//
//  RESTAmIMullvad.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2025-10-13.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public struct RESTAmIMullvad {
    private let timeoutInterval: TimeInterval = 15
    private let urlSession: URLSessionProtocol
    private let relayCache: CachedRelays

    public init(urlSession: URLSessionProtocol, relayCache: CachedRelays) {
        self.urlSession = urlSession
        self.relayCache = relayCache
    }

    public func fetchCurrentLocationIdentifier() async throws -> REST.LocationIdentifier? {
        // Safe to unwrap since it's a constant.
        let amIUrl = URL(string: REST.amIMullvadHostname).unsafelyUnwrapped

        let amIData = try await urlSession.data(for: URLRequest(url: amIUrl, timeoutInterval: timeoutInterval))
        let amIServerLocation = try JSONDecoder().decode(REST.ServerLocation.self, from: amIData.0)

        let amIId = relayCache.relays.locations.first { (locationId, location) in
            (amIServerLocation.country == location.country) && (amIServerLocation.city == location.city)
        }?.key

        return REST.LocationIdentifier(rawValue: amIId ?? "")
    }

    public func fetchCurrentRelayConstraint() async throws -> RelayConstraint<UserSelectedRelays> {
        let locationIdentifier = try await fetchCurrentLocationIdentifier()

        return .only(
            UserSelectedRelays(
                locations: [.country(String(locationIdentifier?.country ?? "se"))]
            )
        )
    }
}
