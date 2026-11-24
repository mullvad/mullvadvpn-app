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

private struct CustomBackButtonContainerView<Content: View>: View {
    @Environment(\.dismiss) var dismiss

    let content: () -> Content
    var body: some View {
        content()
            .navigationBarBackButtonHidden(true)
            .toolbar {
                ToolbarItem(placement: .navigation) {
                    Button {
                        dismiss()
                    } label: {
                        Image.mullvadIconBack
                    }
                    .accessibilityIdentifier(.navigationBackButton)
                }.hideLiquidGlassEffect()
            }
    }
}

extension View {
    /// Adjust this view so that it does not have a system-provided Back button, but
    /// instead has a custom Mullvad-style Back button.
    /// This is to avoid the Liquid Glass look of system-provided navigation controls.
    @ViewBuilder
    func withCustomBackButton() -> some View {
        if #available(iOS 26.0, *) {
            CustomBackButtonContainerView { self }
        } else {
            self
        }

    }
}
