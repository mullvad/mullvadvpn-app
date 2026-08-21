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

struct ActionBox<Style: ToggleStyle>: View {
    @State var isChecked: Bool

    let label: String
    let toggleStyle: Style

    var didToggle: (Bool) -> Void

    var body: some View {
        Toggle(NSLocalizedString(label, comment: ""), isOn: $isChecked)
            .padding(UIMetrics.ActionBox.padding)
            .border(Color.MullvadActionBox.border)
            .cornerRadius(UIMetrics.ActionBox.cornerRadius)
            .toggleStyle(toggleStyle)
            .onChange(of: isChecked) { _, newValue in
                didToggle(newValue)
            }
    }
}
