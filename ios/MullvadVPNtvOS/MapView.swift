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

struct MapView: View {
    @Environment(Navigation.self) private var navigation: Navigation
    var body: some View {
        VStack {
            HeaderBar()
                .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: 100)
                .background(Color.mullvadDangerColor)
                Spacer()
            Text("Congratulations you are logged in !")
        }
        .background(Color.mullvadBackground)
    }
}

#Preview {
    MapView().environment(Navigation())
}

