//
//  RelayWithLocation.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-05-17.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

struct RelayWithLocation<T: AnyRelay> {
    let relay: T
    let serverLocation: Location

    func matches(location: RelayLocation) -> Bool {
        return switch location {
        case let .country(countryCode):
            serverLocation.countryCode == countryCode

        case let .city(countryCode, cityCode):
            serverLocation.countryCode == countryCode &&
                serverLocation.cityCode == cityCode

        case let .hostname(countryCode, cityCode, hostname):
            serverLocation.countryCode == countryCode &&
                serverLocation.cityCode == cityCode &&
                relay.hostname == hostname
        }
    }
}
