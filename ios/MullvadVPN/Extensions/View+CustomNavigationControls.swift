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

extension View {
    /// Adjust this view so that it does not have a system-provided Back button, but
    /// instead has a custom Mullvad-style Back button, which executes an arbitrary
    /// action (which should pop the top view from the navigation stack).
    /// This is to avoid the Liquid Glass look of system-provided navigation controls.
    func withCustomBackButton(_ action: @escaping (() -> Void)) -> some View {
        self
            .navigationBarBackButtonHidden(true)
            .toolbar {
                ToolbarItem(placement: .navigationBarLeading) {
                    Button(
                        action: action
                    ) {
                        Image.mullvadIconBack
                    }
                    .accessibilityIdentifier(.navigationBackButton)
                }.hideLiquidGlassEffect()
            }
    }
}
