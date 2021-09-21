//
//  IPAddressRange+Codable.swift
//  PacketTunnel
//
//  Created by pronebird on 05/01/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import Foundation
import struct WireGuardKit.IPAddressRange

extension IPAddressRange: Codable {
    public func encode(to encoder: Encoder) throws {
        var container = encoder.singleValueContainer()

        try container.encode(self.stringRepresentation)
    }

    public init(from decoder: Decoder) throws {
        let container = try decoder.singleValueContainer()
        let value = try container.decode(String.self)

        if let ipAddressRange = IPAddressRange(from: value) {
            self = ipAddressRange
        } else {
            let context = DecodingError.Context(
                codingPath: container.codingPath,
                debugDescription: "Invalid IPAddressRange representation"
            )
            throw DecodingError.dataCorrupted(context)
        }
    }
}
