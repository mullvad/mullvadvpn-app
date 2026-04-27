//
//  TrafficStatus.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

struct TrafficStatus {
    var speed: TrafficSpeed
    var data: TrafficData

    init(speed: TrafficSpeed = .zero, data: TrafficData = .zero) {
        self.speed = speed
        self.data = data
    }
}
