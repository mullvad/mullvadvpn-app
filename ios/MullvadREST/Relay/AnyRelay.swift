//
//  AnyRelay.swift
//  MullvadREST
//
//  Created by Jon Petersson on 2024-01-31.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import Network

// TODO: move this elsewhere
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

extension REST.LocationIdentifier: Hashable {
    public static func ==(lhs: REST.LocationIdentifier, rhs: REST.LocationIdentifier) -> Bool {
        lhs.rawValue == rhs.rawValue
    }
    
    public func hash(into hasher: inout Hasher) {
        hasher.combine(rawValue)
    }
}

extension REST.LocationIdentifier: Codable {
    
    enum ParsingError: Error {
        case malformed
    }
    
    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        guard let parsed = Self.parse(try container.decode(String.self))  else {
            throw ParsingError.malformed
        }
        (country, city) = parsed
    }
    
    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(rawValue)
    }
}

public protocol AnyRelay {
    var hostname: String { get }
    var owned: Bool { get }
    var location: REST.LocationIdentifier { get }
    var provider: String { get }
    var weight: UInt64 { get }
    var active: Bool { get }
    var includeInCountry: Bool { get }
    var daita: Bool? { get }

    func override(ipv4AddrIn: IPv4Address?, ipv6AddrIn: IPv6Address?) -> Self
}

extension REST.ServerRelay: AnyRelay {}
extension REST.BridgeRelay: AnyRelay {
    public func override(ipv4AddrIn: IPv4Address?, ipv6AddrIn: IPv6Address?) -> REST.BridgeRelay {
        override(ipv4AddrIn: ipv4AddrIn)
    }
}
