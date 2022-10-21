//
//  RelayConstraint.swift
//  MullvadTypes
//
//  Created by pronebird on 21/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let anyConstraint = "any"

public enum RelayConstraint<T>: Codable, Equatable,
    CustomDebugStringConvertible where T: Codable & Equatable
{
    case any
    case only(T)

    public var value: T? {
        if case let .only(value) = self {
            return value
        } else {
            return nil
        }
    }

    public var debugDescription: String {
        var output = "RelayConstraint."
        switch self {
        case .any:
            output += "any"
        case let .only(value):
            output += "only(\(String(reflecting: value)))"
        }
        return output
    }

    private struct OnlyRepr: Codable {
        var only: T
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()

        let decoded = try? container.decode(String.self)
        if decoded == anyConstraint {
            self = .any
        } else {
            let onlyVariant = try container.decode(OnlyRepr.self)

            self = .only(onlyVariant.only)
        }
    }

    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        switch self {
        case .any:
            try container.encode(anyConstraint)
        case let .only(inner):
            try container.encode(OnlyRepr(only: inner))
        }
    }
}
