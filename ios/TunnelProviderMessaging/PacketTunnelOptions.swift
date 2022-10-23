//
//  PacketTunnelOptions.swift
//  TunnelProviderMessaging
//
//  Created by pronebird on 22/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import RelaySelector

public struct PacketTunnelOptions {
    /// Keys for options dictionary
    private enum Keys: String {
        /// Option key that holds the `NSData` value with `RelaySelectorResult`
        /// encoded using `JSONEncoder`.
        /// Used for passing the pre-selected relay in the GUI process to the Packet tunnel process.
        case relaySelectorResult = "relay-selector-result"

        /// Option key that holds the `NSNumber` value, which is when set to `1` indicates that
        /// the tunnel was started by the system.
        /// System automatically provides that flag to the tunnel.
        case isOnDemand = "is-on-demand"
    }

    private var _rawOptions: [String: NSObject]

    public func rawOptions() -> [String: NSObject] {
        return _rawOptions
    }

    public init() {
        _rawOptions = [:]
    }

    public init(rawOptions: [String: NSObject]) {
        _rawOptions = rawOptions
    }

    public func getSelectorResult() throws -> RelaySelectorResult? {
        guard let data = _rawOptions[Keys.relaySelectorResult.rawValue] as? Data else { return nil }

        return try Self.decode(RelaySelectorResult.self, data)
    }

    public mutating func setSelectorResult(_ value: RelaySelectorResult) throws {
        _rawOptions[Keys.relaySelectorResult.rawValue] = try Self.encode(value) as NSData
    }

    public func isOnDemand() -> Bool {
        return _rawOptions[Keys.isOnDemand.rawValue] as? Int == .some(1)
    }

    /// Encode custom parameter value
    private static func encode<T: Codable>(_ value: T) throws -> Data {
        return try JSONEncoder().encode(value)
    }

    /// Decode custom parameter value
    private static func decode<T: Codable>(_ type: T.Type, _ data: Data) throws -> T {
        return try JSONDecoder().decode(T.self, from: data)
    }
}
