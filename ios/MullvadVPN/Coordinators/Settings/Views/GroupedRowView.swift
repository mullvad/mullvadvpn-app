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

struct GroupedRowView<Content: View>: View {
    let content: Content

    init(@ViewBuilder _ content: () -> Content) {
        self.content = content()
    }

    var body: some View {
        VStack(spacing: 0) {
            content
        }
        .background(Color(UIColor.primaryColor))
        .cornerRadius(UIMetrics.SettingsRowView.cornerRadius)
    }
}

#Preview("GroupedRowView") {
    StatefulPreviewWrapper(true) { value in
        GroupedRowView {
            SwitchRowView(isOn: value, text: "Enable")
        }
    }
}
