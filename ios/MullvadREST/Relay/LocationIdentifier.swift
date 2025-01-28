//
//  LocationIdentifier.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-01-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

extension REST {
    // locations are currently always "aa-bbb" for some country code aa and city code bbb. Should this change, this type can be extended.
    public struct LocationIdentifier: Sendable {
        public let country: Substring
        public let city: Substring

        fileprivate static func parse(_ input: String) -> (Substring, Substring)? {
            let components = input.split(separator: "-")
            guard components.count == 2 else { return nil }
            return (components[0], components[1])
        }
    }
}

extension REST.LocationIdentifier: RawRepresentable {
    public var rawValue: String { country.base }

    public init?(rawValue: String) {
        guard let parsed = Self.parse(rawValue) else { return nil }
        (country, city) = parsed
    }
}

extension REST.LocationIdentifier: ExpressibleByStringLiteral {
    public init(stringLiteral value: StringLiteralType) {
        guard let parsed = Self.parse(value) else {
            // this is ugly, but it will only ever be called for
            // code initialised from a literal in code, and
            // never from real-world input, so it'll have to do.
            fatalError("Invalid LocationIdentifier: \(value)")
        }
        (country, city) = parsed
    }
}

// Allow LocationIdentifier to code to/from JSON Strings
extension REST.LocationIdentifier: Codable {
    enum ParsingError: Error {
        case malformed
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        guard let parsed = Self.parse(try container.decode(String.self)) else {
            throw ParsingError.malformed
        }
        (country, city) = parsed
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(rawValue)
    }
}

// As the location's values are Substrings of the same String, to which they maintain references, we use the base String for holistic operations such as equality and hashing
extension REST.LocationIdentifier: Hashable {
    public static func == (lhs: REST.LocationIdentifier, rhs: REST.LocationIdentifier) -> Bool {
        lhs.rawValue == rhs.rawValue
    }

    public func hash(into hasher: inout Hasher) {
        hasher.combine(rawValue)
    }
}
