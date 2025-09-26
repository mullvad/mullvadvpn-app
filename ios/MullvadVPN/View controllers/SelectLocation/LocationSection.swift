//
//  LocationSection.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum LocationSection: String, Hashable, CaseIterable, CellIdentifierProtocol, Sendable {
    case customLists
    case allLocations

    var header: String {
        switch self {
        case .customLists:
            return NSLocalizedString("Custom lists", comment: "")
        case .allLocations:
            return NSLocalizedString("All locations", comment: "")
        }
    }

    var footer: String {
        switch self {
        case .customLists:
            return NSLocalizedString("To create a custom list, tap on \"...\" ", comment: "")
        case .allLocations:
            return NSLocalizedString("No matching relays found, check your filter settings.", comment: "")
        }
    }

    var cellClass: AnyClass {
        LocationCell.self
    }
}
