//
//  LocationSection.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-02-05.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation

enum LocationSection: String, Hashable, CaseIterable, CellIdentifierProtocol, Sendable {
    case main

    var cellClass: AnyClass {
        LocationCell.self
    }
}
