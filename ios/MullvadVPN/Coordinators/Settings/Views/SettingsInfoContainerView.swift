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

struct SettingsInfoContainerView<Content: View>: View {
    let content: Content

    init(@ViewBuilder _ content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        ScrollView {
            VStack {
                content
                    .padding(.top, UIMetrics.TableView.emptyHeaderHeight)
                    .padding(.bottom, UIMetrics.contentInsets.bottom)
            }
        }
        .accessibilityIdentifier(.settingsInfoView)
        .background(Color(.secondaryColor))
    }
}

#Preview {
    SettingsInfoContainerView {
        SettingsDAITAView(tunnelViewModel: MockDAITATunnelSettingsViewModel())
    }
}
