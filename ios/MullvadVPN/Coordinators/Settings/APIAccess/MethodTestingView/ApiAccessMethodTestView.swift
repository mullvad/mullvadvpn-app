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

struct ApiAccessMethodTestView: View {
    let status: ApiAccessMethodTestState
    let onStart: () -> Void
    let onStop: () -> Void

    var body: some View {
        VStack(spacing: 8) {
            MullvadButton(
                text: "Test method", style: .primary,
                action: {
                    onStart()
                }
            )
            .accessibilityIdentifier(AccessibilityIdentifier.accessMethodTestButton)
            .disabled(status == .testing)

            Text("Performs a connection test to a Mullvad API server via this access method.")
                .font(.mullvadTiny)
                .foregroundStyle(Color.mullvadTextSecondary)
                .padding(.horizontal, 16.0)

            MethodTestingStatusView(
                status: status,
                onStop: {
                    onStop()
                }
            )
            .padding(.vertical, 24.0)
        }
    }
}
