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

struct MullvadListSectionFooter: View {
    let title: LocalizedStringKey
    var body: some View {
        Text(title)
            .font(.mullvadMini)
            .foregroundStyle(Color.mullvadTextPrimary.opacity(0.6))
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(.bottom, 24)
    }
}
#Preview {
    MullvadListSectionFooter(title: "Custom lists")
}
