//
//  SettingsParser.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-22.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct VersionHeader: Codable {
    var version: Int
}

struct Payload<T: Codable>: Codable {
    var data: T
}

struct VersionedPayload<T: Codable>: Codable {
    var version: Int
    var data: T
}

struct SettingsParser {
    /// The decoder used to decode values.
    private let decoder: JSONDecoder

    /// The encoder used to encode values.
    private let encoder: JSONEncoder

    init(
        decoder: JSONDecoder,
        encoder: JSONEncoder
    ) {
        self.decoder = decoder
        self.encoder = encoder
    }

    func producePayload<T: Encodable>(_ payload: VersionedPayload<T>) throws -> Data {
        try encoder.encode(payload)
    }

    func produceUnversionedPayload<T: Encodable>(_ payload: T) throws -> Data {
        try encoder.encode(payload)
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

    /// Returns payload type holding the given type.
    func parsePayload<T: Codable>(
        as type: T.Type,
        from data: Data
    ) throws -> T {
        return try decoder.decode(Payload<T>.self, from: data).data
    }
}
