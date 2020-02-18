//
//  GeoLocation.swift
//  MullvadVPN
//
//  Created by pronebird on 12/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

struct GeoLocation: Codable, Equatable {
    var country: String
    var city: String
    var latitude: Double
    var longitude: Double
}
