//
//  StoredRelays.swift
//  MullvadVPNUITests
//
//  Created by Jon Petersson on 2024-09-09.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

/// A struct that represents the relay cache on disk
public struct StoredRelays: Codable, Equatable {
    /// E-tag returned by server
    public let etag: String?

    /// The raw relay JSON data stored within the cache entry
    public let rawData: Data

    /// The date when this cache was last updated
    public let updatedAt: Date

    /// Thread-safe lazy cache for the deserialized relay response.
    /// Deserialization happens exactly once per `StoredRelays` instance.
    private let cache = DeserializationCache()

    /// `CachedRelays` representation, deserialized on first access and cached thereafter.
    public var cachedRelays: CachedRelays {
        get throws {
            try cache.get {
                let relays = try REST.Coding.makeJSONDecoder().decode(
                    REST.ServerRelaysResponse.self,
                    from: rawData
                )
                return CachedRelays(etag: etag, relays: relays, updatedAt: updatedAt)
            }
        }
    }

    public init(etag: String? = nil, rawData: Data, updatedAt: Date) throws {
        self.etag = etag
        self.rawData = rawData
        self.updatedAt = updatedAt
        // Eagerly deserialize so failures surface at construction time
        // and the UI is never blocked on decoding.
        let relays = try REST.Coding.makeJSONDecoder().decode(
            REST.ServerRelaysResponse.self,
            from: rawData
        )
        cache.set(.success(CachedRelays(etag: etag, relays: relays, updatedAt: updatedAt)))
    }

    public init(cachedRelays: CachedRelays) throws {
        etag = cachedRelays.etag
        rawData = try REST.Coding.makeJSONEncoder().encode(cachedRelays.relays)
        updatedAt = cachedRelays.updatedAt
        // Pre-populate cache with the already-known value.
        cache.set(.success(cachedRelays))
    }

    // MARK: - Codable

    private enum CodingKeys: String, CodingKey {
        case etag, rawData, updatedAt
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        etag = try container.decodeIfPresent(String.self, forKey: .etag)
        rawData = try container.decode(Data.self, forKey: .rawData)
        updatedAt = try container.decode(Date.self, forKey: .updatedAt)
        // Eagerly deserialize relay data so the result is cached before
        // this value reaches the actor's cooperative thread. The
        // DeserializationCache.get() NSLock + JSON parsing would otherwise
        // block a cooperative thread during relay selection.
        let result = Result {
            try REST.Coding.makeJSONDecoder().decode(
                REST.ServerRelaysResponse.self,
                from: rawData
            )
        }.map { CachedRelays(etag: etag, relays: $0, updatedAt: updatedAt) }
        cache.set(result)
    }

    // MARK: - Equatable

    public static func == (lhs: StoredRelays, rhs: StoredRelays) -> Bool {
        lhs.etag == rhs.etag && lhs.rawData == rhs.rawData && lhs.updatedAt == rhs.updatedAt
    }
}

/// Thread-safe cache that stores either a successfully computed value or a thrown error.
/// Once resolved, subsequent calls to `get` return the cached result without re-invoking the closure.
private final class DeserializationCache: @unchecked Sendable {
    private let lock = NSLock()
    private var result: Result<CachedRelays, Error>?

    func get(_ compute: () throws -> CachedRelays) throws -> CachedRelays {
        lock.lock()
        defer { lock.unlock() }

        if let result {
            return try result.get()
        }

        let newResult = Result { try compute() }
        result = newResult
        return try newResult.get()
    }

    func set(_ value: Result<CachedRelays, Error>) {
        lock.lock()
        defer { lock.unlock() }
        result = value
    }
}
