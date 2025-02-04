//
//  RelayWithDistance.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
struct RelayWithDistance<T: AnyRelay> {
    let relay: T
    let distance: Double
}
