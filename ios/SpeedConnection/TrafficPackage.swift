//
//  TrafficPackage.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation

enum InterfaceType {
    case wifi
    case cellular
    case vpn
    case loopback
    case other
}

struct TrafficPackage {
    var wifi = TrafficData()
    var cellular = TrafficData()
    var vpn = TrafficData()
}
