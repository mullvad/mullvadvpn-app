//
//  TransportLayer.swift
//  MullvadTypes
//
//  Created by Marco Nikic on 2023-11-24.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum TransportLayer: Codable, Sendable {
    case udp
    case tcp
}
