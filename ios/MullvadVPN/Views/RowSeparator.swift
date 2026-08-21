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

struct RowSeparator: View {
    let color: Color
    let edgeInsets: EdgeInsets

    init(color: Color = Color(.secondaryColor), edgeInsets: EdgeInsets = .init()) {
        self.color = color
        self.edgeInsets = edgeInsets
    }

    var body: some View {
        color
            .frame(height: UIMetrics.TableView.separatorHeight)
            .padding(edgeInsets)
    }
}

#Preview {
    RowSeparator(color: Color(.primaryColor))
}
