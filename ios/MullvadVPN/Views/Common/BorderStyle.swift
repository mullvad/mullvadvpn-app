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

enum BorderStyle {
    case normal
    case focused
    case error
    case none

    var color: Color {
        switch self {
        case .normal:
            Color.MullvadTextField.border
        case .focused:
            Color.MullvadTextField.borderFocused
        case .error:
            Color.MullvadTextField.borderError
        case .none:
            Color.MullvadTextField.borderDisabled
        }
    }

    var lineWidth: CGFloat {
        switch self {
        case .focused, .error:
            2.0
        case .none:
            0.0
        default:
            1.0
        }
    }
}
