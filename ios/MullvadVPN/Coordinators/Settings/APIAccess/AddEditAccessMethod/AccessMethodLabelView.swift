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

struct AccessMethodLabelView: View {
    let title: LocalizedStringKey
    let textColor: Color
    var body: some View {
        HStack {
            Text(title)
                .font(.mullvadSmallSemiBold)
                .foregroundStyle(textColor)
                .multilineTextAlignment(.leading)
            Spacer()
        }
        .padding(.vertical, 8)
        .padding(.trailing, 16)
        .frame(minHeight: UIMetrics.LocationList.cellMinHeight)
    }
}
