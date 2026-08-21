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

extension UIEdgeInsets {
    /// Returns directional edge insets mapping left edge to leading and right edge to trailing.
    var toDirectionalInsets: NSDirectionalEdgeInsets {
        NSDirectionalEdgeInsets(
            top: top,
            leading: left,
            bottom: bottom,
            trailing: right
        )
    }

    /// Returns edge insets.
    var toEdgeInsets: EdgeInsets {
        EdgeInsets(toDirectionalInsets)
    }
}
