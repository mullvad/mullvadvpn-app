//
//  SettingsParser.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-22.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

private struct VersionHeader: Codable {
    var version: Int
}

private struct Payload<T: Codable>: Codable {
    var data: T
}

private struct VersionedPayload<T: Codable>: Codable {
    var version: Int
    var data: T
}

struct SettingsParser {
    /// The decoder used to decode values.
    private let decoder: JSONDecoder

    /// The encoder used to encode values.
    private let encoder: JSONEncoder

    init(decoder: JSONDecoder, encoder: JSONEncoder) {
        self.decoder = decoder
        self.encoder = encoder
    }

    /// Produces versioned data encoded as the given type
    func producePayload<T: Codable>(_ payload: T, version: Int) throws -> Data {
        return try encoder.encode(VersionedPayload(version: version, data: payload))
    }

    /// Produces unversioned data encoded as the given type
    func produceUnversionedPayload<T: Codable>(_ payload: T) throws -> Data {
        return try encoder.encode(payload)
    }

    /// Returns settings version if found inside the stored data.
    func parseVersion(data: Data) throws -> Int {
        let header = try decoder.decode(VersionHeader.self, from: data)

        return header.version
    }

    /// Returns unversioned payload parsed as the given type.
    func parseUnversionedPayload<T: Codable>(
        as type: T.Type,
        from data: Data
    ) throws -> T {
        return try decoder.decode(T.self, from: data)
    }

    /// Returns data from versioned payload parsed as the given type.
    func parsePayload<T: Codable>(
        as type: T.Type,
        from data: Data
    ) throws -> T {
        return try decoder.decode(Payload<T>.self, from: data).data
    }
}
