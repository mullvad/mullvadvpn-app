//
//  LocalNetworkIPs.swift
//  MullvadTypes
//
//  Created by Mojgan on 2024-07-26.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum LocalNetworkIPs: String {
    case gatewayAddress = "10.64.0.1"
    case defaultRouteIpV4 = "0.0.0.0"
    case defaultRouteIpV6 = "::"
}
