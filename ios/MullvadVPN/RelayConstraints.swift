//
//  RelayConstraint.swift
//  MullvadVPN
//
//  Created by pronebird on 10/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let kRelayConstraintAnyRepr = "any"

enum RelayConstraint<T>: Codable, Equatable where T: Codable & Equatable {
    case any
    case only(T)

    var value: T? {
        if case let .only(value) = self {
            return value
        } else {
            return nil
        }
    }

    private struct OnlyRepr: Codable {
        var only: T
    }

    init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        let decoded = try? container.decode(String.self)
        if decoded == kRelayConstraintAnyRepr {
            self = .any
        } else {
            let onlyVariant = try container.decode(OnlyRepr.self)

            self = .only(onlyVariant.only)
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch self {
        case .any:
            try container.encode(kRelayConstraintAnyRepr)
        case let .only(inner):
            try container.encode(OnlyRepr(only: inner))
        }
    }
}

extension RelayConstraint: CustomDebugStringConvertible {
    var debugDescription: String {
        var output = "RelayConstraint."
        switch self {
        case .any:
            output += "any"
        case let .only(value):
            output += "only(\(String(reflecting: value)))"
        }
        return output
    }
}

enum RelayLocation: Codable, Hashable {
    case country(String)
    case city(String, String)
    case hostname(String, String, String)

    init?(dashSeparatedString: String) {
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

    init(from decoder: Decoder) throws {
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

    func encode(to encoder: Encoder) throws {
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
    var ascendants: [RelayLocation] {
        switch self {
        case let .hostname(country, city, _):
            return [.country(country), .city(country, city)]

        case let .city(country, _):
            return [.country(country)]

        case .country:
            return []
        }
    }
}

extension RelayLocation: CustomDebugStringConvertible {
    var debugDescription: String {
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

    var stringRepresentation: String {
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

struct RelayConstraints: Codable, Equatable {
    var location: RelayConstraint<RelayLocation> = .only(.country("se"))
}

extension RelayConstraints: CustomDebugStringConvertible {
    var debugDescription: String {
        var output = "RelayConstraints { "
        output += "location: \(String(reflecting: location))"
        output += " }"
        return output
    }
}
