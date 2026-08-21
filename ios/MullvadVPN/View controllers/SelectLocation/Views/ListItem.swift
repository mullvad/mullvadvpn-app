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

struct ListItem<StatusIndicator: View>: View {
    let title: String
    var subtitle: String?
    var level: Int = 0
    var selected: Bool = false
    @ViewBuilder var statusIndicator: () -> StatusIndicator?

    var body: some View {
        HStack {
            statusIndicator()
                .frame(width: 24, height: 24)

            VStack(alignment: .leading) {
                Text(title)
                    .font(.mullvadSmallSemiBold)
                    .foregroundStyle(selected ? Color.mullvadSuccessColor : Color.mullvadTextPrimary)
                    .multilineTextAlignment(.leading)
                if let subtitle {
                    Text(subtitle)
                        .font(.mullvadMiniSemiBold)
                        .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
                        .multilineTextAlignment(.leading)
                }
            }

            Spacer()
        }
        .padding(.vertical, 8)
        .padding(.leading, CGFloat(16 * level))
        .padding(.trailing, 16)
        .frame(minHeight: UIMetrics.LocationList.cellMinHeight)
    }
}
