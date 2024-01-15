//
//  RelayCache.swift
//  RelayCache
//
//  Created by pronebird on 06/09/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import Network

public protocol RelayCacheProtocol {
    func read() throws -> CachedRelays
    func write(record: CachedRelays) throws
}

public final class RelayCache: RelayCacheProtocol {
    private let fileCache: any FileCacheProtocol<CachedRelays>
    private let ipOverrideRepository: any IPOverrideRepositoryProtocol

    /// Designated initializer
    public init(cacheDirectory: URL, ipOverrideRepository: IPOverrideRepositoryProtocol) {
        fileCache = FileCache(fileURL: cacheDirectory.appendingPathComponent("relays.json", isDirectory: false))
        self.ipOverrideRepository = ipOverrideRepository
    }

    /// Initializer that accepts a custom FileCache implementation. Used in tests.
    init(fileCache: some FileCacheProtocol<CachedRelays>, ipOverrideRepository: some IPOverrideRepositoryProtocol) {
        self.fileCache = fileCache
        self.ipOverrideRepository = ipOverrideRepository
    }

    /// Safely read the cache file from disk using file coordinator and fallback to prebundled
    /// relays in case if the relay cache file is missing.
    public func read() throws -> CachedRelays {
        do {
            let cache = try fileCache.read()
            let relayResponse = apply(overrides: ipOverrideRepository.fetchAll(), to: cache.relays)

            return CachedRelays(relays: relayResponse, updatedAt: cache.updatedAt)
        } catch {
            if error is DecodingError || (error as? CocoaError)?.code == .fileReadNoSuchFile {
                return try readPrebundledRelays()
            } else {
                throw error
            }
        }
    }

    /// Safely write the cache file on disk using file coordinator.
    public func write(record: CachedRelays) throws {
        try fileCache.write(record)
    }

    /// Read pre-bundled relays file from disk.
    private func readPrebundledRelays() throws -> CachedRelays {
        guard let prebundledRelaysFileURL = Bundle(for: Self.self).url(forResource: "relays", withExtension: "json")
        else { throw CocoaError(.fileNoSuchFile) }

        let data = try Data(contentsOf: prebundledRelaysFileURL)
        let relays = try REST.Coding.makeJSONDecoder().decode(REST.ServerRelaysResponse.self, from: data)

        return CachedRelays(
            relays: relays,
            updatedAt: Date(timeIntervalSince1970: 0)
        )
    }

    private func apply(
        overrides: [IPOverride],
        to relyResponse: REST.ServerRelaysResponse
    ) -> REST.ServerRelaysResponse {
        let wireguard = relyResponse.wireguard
        let bridge = relyResponse.bridge

        let wireguardRelays = wireguard.relays.map { relay in
            return apply(overrides: overrides, to: relay)
        }
        let bridgeRelays = bridge.relays.map { relay in
            return apply(overrides: overrides, to: relay)
        }

        return REST.ServerRelaysResponse(
            locations: relyResponse.locations,
            wireguard: REST.ServerWireguardTunnels(
                ipv4Gateway: wireguard.ipv4Gateway,
                ipv6Gateway: wireguard.ipv6Gateway,
                portRanges: wireguard.portRanges,
                relays: wireguardRelays
            ),
            bridge: REST.ServerBridges(
                shadowsocks: bridge.shadowsocks,
                relays: bridgeRelays
            )
        )
    }

    private func apply<T: AnyRelay>(overrides: [IPOverride], to relay: T) -> T {
        if let override = overrides.first(where: { host in
            host.hostname == relay.hostname
        }) {
            return relay.copyWith(
                ipv4AddrIn: override.ipv4Address,
                ipv6AddrIn: override.ipv6Address
            )
        }

        return relay
    }
}
