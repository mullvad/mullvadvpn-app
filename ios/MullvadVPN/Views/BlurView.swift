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

/// Blurred (background) view using a `UIBlurEffect`.
struct BlurView: View {
    var style: UIBlurEffect.Style

    var body: some View {
        VisualEffectView(effect: UIBlurEffect(style: style))
            .opacity(0.8)
    }
}
