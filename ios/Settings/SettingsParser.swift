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

public struct SettingsParser {
    /// The decoder used to decode values.
    private let decoder: JSONDecoder

    /// The encoder used to encode values.
    private let encoder: JSONEncoder

    public init(decoder: JSONDecoder, encoder: JSONEncoder) {
        self.decoder = decoder
        self.encoder = encoder
    }

    /// Produces versioned data encoded as the given type
    public func producePayload(_ payload: some Codable, version: Int) throws -> Data {
        try encoder.encode(VersionedPayload(version: version, data: payload))
    }

    /// Produces unversioned data encoded as the given type
    public func produceUnversionedPayload(_ payload: some Codable) throws -> Data {
        try encoder.encode(payload)
    }

    /// Returns settings version if found inside the stored data.
    public func parseVersion(data: Data) throws -> Int {
        let header = try decoder.decode(VersionHeader.self, from: data)

        return header.version
    }

    /// Returns unversioned payload parsed as the given type.
    public func parseUnversionedPayload<T: Codable>(
        as type: T.Type,
        from data: Data
    ) throws -> T {
        try decoder.decode(T.self, from: data)
    }

    /// Returns data from versioned payload parsed as the given type.
    public func parsePayload<T: Codable>(
        as type: T.Type,
        from data: Data
    ) throws -> T {
        try decoder.decode(Payload<T>.self, from: data).data
    }
}
