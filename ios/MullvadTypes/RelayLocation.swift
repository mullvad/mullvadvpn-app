//
//  RelayLocation.swift
//  MullvadTypes
//
//  Created by pronebird on 21/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum RelayLocation: Codable, Hashable, CustomDebugStringConvertible {
    case country(String)
    case city(String, String)
    case hostname(String, String, String)

    public init?(dashSeparatedString: String) {
        let components = dashSeparatedString.split(separator: "-", maxSplits: 2).map(String.init)

        switch components.count {
        case 1:
            self = .country(components[0])
        case 2:
            self = .city(components[0], components[1])
        case 3:
            self = .hostname(components[0], components[1], components[2])
        default:
            return nil
        }
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let components = try container.decode([String].self)

        switch components.count {
        case 1:
            self = .country(components[0])
        case 2:
            self = .city(components[0], components[1])
        case 3:
            self = .hostname(components[0], components[1], components[2])
        default:
            throw DecodingError.dataCorruptedError(
                in: container,
                debugDescription: "Invalid enum representation"
            )
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch self {
        case let .country(code):
            try container.encode([code])

        case let .city(countryCode, cityCode):
            try container.encode([countryCode, cityCode])

        case let .hostname(countryCode, cityCode, hostname):
            try container.encode([countryCode, cityCode, hostname])
        }
    }

    /// A list of `RelayLocation` items preceding the given one in the relay tree
    public var ascendants: [RelayLocation] {
        switch self {
        case let .hostname(country, city, _):
            return [.country(country), .city(country, city)]

        case let .city(country, _):
            return [.country(country)]

        case .country:
            return []
        }
    }

    public var debugDescription: String {
        var output = "RelayLocation."

        switch self {
        case let .country(country):
            output += "country(\(String(reflecting: country)))"

        case let .city(country, city):
            output += "city(\(String(reflecting: country)), \(String(reflecting: city)))"

        case let .hostname(country, city, host):
            output += "hostname(\(String(reflecting: country)), " +
                "\(String(reflecting: city)), " +
                "\(String(reflecting: host)))"
        }

        return output
    }

    public var stringRepresentation: String {
        switch self {
        case let .country(country):
            return country
        case let .city(country, city):
            return "\(country)-\(city)"
        case let .hostname(country, city, host):
            return "\(country)-\(city)-\(host)"
        }
    }
}
