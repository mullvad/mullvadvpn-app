// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import CoreLocation
import Foundation

public struct Location: Codable, Equatable, Hashable, Sendable {
    public var country: String
    public var countryCode: String
    public var city: String
    public var cityCode: String
    public var latitude: Double
    public var longitude: Double

    public var geoCoordinate: CLLocationCoordinate2D {
        CLLocationCoordinate2D(latitude: latitude, longitude: longitude)
    }

    public init(
        country: String,
        countryCode: String,
        city: String,
        cityCode: String,
        latitude: Double,
        longitude: Double
    ) {
        self.country = country
        self.countryCode = countryCode
        self.city = city
        self.cityCode = cityCode
        self.latitude = latitude
        self.longitude = longitude
    }
}
