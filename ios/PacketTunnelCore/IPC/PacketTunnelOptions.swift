//
//  PacketTunnelOptions.swift
//  PacketTunnelCore
//
//  Created by pronebird on 22/08/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation

public struct PacketTunnelOptions {
    /// Keys for options dictionary
    private enum Keys: String {
        /// Option key that holds serialized`SelectedRelay` value encoded using `JSONEncoder`.
        /// Used for passing the pre-selected relay in the GUI process to the Packet tunnel process.
        case selectedRelay = "selected-relay"

        /// Option key that holds the `NSNumber` value, which is when set to `1` indicates that
        /// the tunnel was started by the system.
        /// System automatically provides that flag to the tunnel.
        case isOnDemand = "is-on-demand"
    }

    private var _rawOptions: [String: NSObject]

    public func rawOptions() -> [String: NSObject] {
        _rawOptions
    }

    public init() {
        _rawOptions = [:]
    }

    public init(rawOptions: [String: NSObject]) {
        _rawOptions = rawOptions
    }

    public func getSelectedRelay() throws -> SelectedRelay? {
        guard let data = _rawOptions[Keys.selectedRelay.rawValue] as? Data else { return nil }

        return try Self.decode(SelectedRelay.self, data)
    }

    public mutating func setSelectedRelay(_ value: SelectedRelay) throws {
        _rawOptions[Keys.selectedRelay.rawValue] = try Self.encode(value) as NSData
    }

    public func isOnDemand() -> Bool {
        _rawOptions[Keys.isOnDemand.rawValue] as? Int == 1
    }

    /// Encode custom parameter value
    private static func encode(_ value: some Codable) throws -> Data {
        try JSONEncoder().encode(value)
    }

    /// Decode custom parameter value
    private static func decode<T: Codable>(_ type: T.Type, _ data: Data) throws -> T {
        try JSONDecoder().decode(T.self, from: data)
    }
}
