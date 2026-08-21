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

struct BulletPointText: View {
    let text: String
    let bullet = "•"

    var body: some View {
        HStack(alignment: .firstTextBaseline) {
            Text(bullet)
                .font(.body)
                .foregroundColor(UIColor.secondaryTextColor.color)
            Text(text)
                .font(.body)
                .foregroundColor(UIColor.secondaryTextColor.color)
                .lineLimit(nil)
        }
    }
}
