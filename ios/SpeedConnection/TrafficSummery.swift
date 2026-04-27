//
//  TrafficSummery.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation

struct TrafficSummery {
    let wifi: TrafficStatus
    let cellular: TrafficStatus
    let vpn: TrafficStatus?

    init(
        wifi: TrafficStatus = TrafficStatus(),
        cellular: TrafficStatus = TrafficStatus(),
        vpn: TrafficStatus? = nil
    ) {
        self.wifi = wifi
        self.cellular = cellular
        self.vpn = vpn
    }

    var speed: TrafficSpeed {
        if let vpn = vpn, !vpn.speed.isZero {
            return vpn.speed
        } else {
            return wifi.speed + cellular.speed
        }
    }

    var data: TrafficData {
        if let vpn = vpn, !vpn.data.isZero {
            return vpn.data
        } else {
            return wifi.data + cellular.data
        }
    }

    static func make(_ old: TrafficPackage, new: TrafficPackage, interval: TimeInterval) -> Self {
        TrafficSummery(
            wifi: TrafficStatus(
                speed: TrafficSpeed(
                    old: old.wifi,
                    new: new.wifi,
                    interval: interval),
                data: new.wifi),
            cellular: TrafficStatus(
                speed: TrafficSpeed(
                    old: old.cellular,
                    new: new.cellular,
                    interval: interval),
                data: new.cellular),
            vpn: TrafficStatus(
                speed: TrafficSpeed(
                    old: old.vpn,
                    new: new.vpn,
                    interval: interval),
                data: new.vpn))
    }

}
