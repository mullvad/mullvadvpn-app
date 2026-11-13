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

struct DoneBarButton: ViewModifier {
    var accessibilityIdentifier: AccessibilityIdentifier?
    var isDisabled: Bool
    let action: () -> Void

    func body(content: Content) -> some View {
        content
            .toolbar {
                ToolbarItem(placement: .topBarTrailing) {
                    Button(action: action) {
                        Image(.iconTick)
                            .renderingMode(.template)
                    }
                    .tint(UIColor.NavigationBar.buttonColor.color)
                    .disabled(isDisabled)
                    .accessibilityIdentifier(accessibilityIdentifier)
                }
                .hideLiquidGlassEffect()
            }
    }
}

extension View {
    func doneBarButton(
        accessibilityIdentifier: AccessibilityIdentifier? = nil,
        isDisabled: Bool = false,
        _ action: @escaping () -> Void
    ) -> some View {
        modifier(
            DoneBarButton(
                accessibilityIdentifier: accessibilityIdentifier,
                isDisabled: isDisabled,
                action: action
            )
        )
    }
}
