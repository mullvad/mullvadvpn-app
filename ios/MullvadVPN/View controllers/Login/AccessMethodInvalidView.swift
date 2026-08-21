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

struct AccessMethodInvalidView: View {
    var didPressButton: () -> Void

    var body: some View {
        ZStack {
            VStack(spacing: 16) {
                VStack(alignment: .leading, spacing: 8) {
                    Text("Custom API access method is invalid")
                        .font(.mullvadSmallSemiBold)
                    Text("Please update it or enable a different one to be able to reach the API using this method.")
                        .font(.mullvadTinySemiBold)
                }
                MullvadButton(text: "API access methods", style: .primary) {
                    didPressButton()
                }
            }
            .padding(UIMetrics.Dashboard.padding)
        }
        .background(Color.MullvadDashboard.background)
        .foregroundStyle(Color.white)
        .cornerRadius(UIMetrics.Dashboard.cornerRadius)
    }
}

#Preview {
    AccessMethodInvalidView {
        print("Pressed button")
    }
}
