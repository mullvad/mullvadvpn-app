// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import SwiftUI
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

    var image: Image {
        switch self {
        case .info:
            .mullvadIconStateOnline
        case .warning:
            .mullvadIconStateIssue
        case .error:
            .mullvadIconStateOffline
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
