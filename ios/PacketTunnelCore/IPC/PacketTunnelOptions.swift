// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadREST

public struct PacketTunnelOptions {
    /// Keys for options dictionary
    private enum Keys: String {
        /// Option key that holds serialized `SelectedRelay` value encoded using `JSONEncoder`.
        /// Used for passing the pre-selected relay in the GUI process to the Packet tunnel process.
        case selectedRelays = "selected-relays"

        /// Option key that holds an `NSNumber` value, which is when set to `1` indicates that the tunnel was started by the system.
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

    public func getSelectedRelays() throws -> SelectedRelays? {
        guard let data = _rawOptions[Keys.selectedRelays.rawValue] as? Data else { return nil }

        return try Self.decode(SelectedRelays.self, data)
    }

    public mutating func setSelectedRelays(_ value: SelectedRelays) throws {
        _rawOptions[Keys.selectedRelays.rawValue] = try Self.encode(value) as NSData
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
