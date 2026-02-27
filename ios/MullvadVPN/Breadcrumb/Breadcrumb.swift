//
//  Breadcrumb.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-03-06.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

enum Breadcrumb: Hashable {
    case info(SettingsNavigationRoute)
    case warning(SettingsNavigationRoute)
    case error(SettingsNavigationRoute)

    var navigationRoute: SettingsNavigationRoute {
        switch self {
        case .info(let route), .warning(let route), .error(let route):
            route
        }
    }

    var icon: UIImage {
        switch self {
        case .info:
            .stateOnline
        case .warning:
            .stateIssue
        case .error:
            .stateOffline
        }
    }
}

extension Set where Element == Breadcrumb {
    var mostSevere: Breadcrumb? {
        first { if case .error = $0 { true } else { false } }
            ?? first { if case .warning = $0 { true } else { false } }
            ?? first { if case .info = $0 { true } else { false } }
    }
}
