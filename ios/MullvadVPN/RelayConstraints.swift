//
//  RelayConstraint.swift
//  MullvadVPN
//
//  Created by pronebird on 10/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let kRelayConstraintAnyRepr = "any"

enum RelayConstraint<T: Codable>: Codable {
    case any
    case only(T)

    var value: T? {
        if case .only(let value) = self {
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
        case .only(let inner):
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
        case .only(let value):
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
                debugDescription: "Invalid enum representation")
        }
    }

    func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch self {
        case .country(let code):
            try container.encode([code])

        case .city(let countryCode, let cityCode):
            try container.encode([countryCode, cityCode])

        case .hostname(let countryCode, let cityCode, let hostname):
            try container.encode([countryCode, cityCode, hostname])
        }
    }

    /// A list of `RelayLocation` items preceding the given one in the relay tree
    var ascendants: [RelayLocation] {
        switch self {
        case .hostname(let country, let city, _):
            return [.country(country), .city(country, city)]

        case .city(let country, _):
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
        case .country(let country):
            output += "country(\(String(reflecting: country)))"

        case .city(let country, let city):
            output += "city(\(String(reflecting: country)), \(String(reflecting: city)))"

        case .hostname(let country, let city, let host):
            output += "hostname(\(String(reflecting: country)), " +
                "\(String(reflecting: city)), " +
                "\(String(reflecting: host)))"
        }

        return output
    }

    var stringRepresentation: String {
        switch self {
        case .country(let country):
            return country
        case .city(let country, let city):
            return "\(country)-\(city)"
        case .hostname(let country, let city, let host):
            return "\(country)-\(city)-\(host)"
        }
    }
}

struct RelayConstraints: Codable {
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
