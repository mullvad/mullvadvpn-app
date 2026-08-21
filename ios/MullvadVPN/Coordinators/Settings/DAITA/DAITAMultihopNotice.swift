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

struct DAITAMultihopNotice: View {
    var body: some View {
        HStack(spacing: 8) {
            Image(.iconInfo)
                .resizable()
                .frame(width: 18, height: 18)
                .foregroundStyle(Color(.primaryTextColor).opacity(0.6))
            Text(NSLocalizedString("Multihop is being used to enable DAITA for your selected location.", comment: ""))
                .font(.mullvadTinySemiBold)
                .foregroundColor(Color(.primaryTextColor).opacity(0.6))
        }
    }
}

#Preview {
    SettingsInfoContainerView {
        DAITAMultihopNotice()
    }
}
