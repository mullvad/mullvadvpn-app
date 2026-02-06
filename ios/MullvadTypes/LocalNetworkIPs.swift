//
//  LocalNetworkIPs.swift
//  MullvadTypes
//
//  Created by Mojgan on 2024-07-26.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation

public enum LocalNetworkIPs: String {
    case gatewayAddressIpV4 = "10.64.0.1"
    case gatewayAddressIpV6 = "fc00::1"
    case defaultRouteIpV4 = "0.0.0.0"
    case defaultRouteIpV6 = "::"
}
