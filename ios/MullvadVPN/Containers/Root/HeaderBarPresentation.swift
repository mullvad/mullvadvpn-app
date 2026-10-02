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

enum HeaderBarStyle: Sendable {
    case transparent, `default`, unsecured, secured

    func backgroundColor() -> Color {
        switch self {
        case .transparent:
            return Color.clear
        case .default:
            return Color.MullvadHeaderBar.defaultBackgroundColor
        case .secured:
            return Color.MullvadHeaderBar.securedBackgroundColor
        case .unsecured:
            return Color.MullvadHeaderBar.unsecuredBackgroundColor
        }
    }
}

struct HeaderBarPresentation: Sendable {
    let style: HeaderBarStyle

    static var `default`: HeaderBarPresentation {
        HeaderBarPresentation(style: .default)
    }
}
