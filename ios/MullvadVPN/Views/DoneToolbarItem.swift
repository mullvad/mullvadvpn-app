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

struct DoneToolbarItem: ToolbarContent {
    var accessibilityIdentifier: AccessibilityIdentifier? = nil
    var isDisabled: Bool = false
    let onDone: () -> Void
    var body: some ToolbarContent {
        ToolbarItem(placement: .confirmationAction) {
            Button(action: onDone) {
                Image(.iconTick)
                    .renderingMode(.template)
            }
            .disabled(isDisabled)
            .buttonStyle(DisabledButtonStyle())
            .accessibilityIdentifier(accessibilityIdentifier)
            .accessibilityLabel(Text("Done"))
            .accessibilityHint(Text("Completes the current action"))
        }
        .hideLiquidGlassEffect()
    }
}
