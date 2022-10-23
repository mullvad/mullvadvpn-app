//
//  Location.swift
//  MullvadTypes
//
//  Created by pronebird on 12/02/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import struct CoreLocation.CLLocationCoordinate2D
import Foundation

public struct Location: Codable, Equatable {
    public var country: String
    public var countryCode: String
    public var city: String
    public var cityCode: String
    public var latitude: Double
    public var longitude: Double

    public var geoCoordinate: CLLocationCoordinate2D {
        return CLLocationCoordinate2D(latitude: latitude, longitude: longitude)
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
