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

struct HeaderBar: View {
    @Environment(Navigation.self) private var navigation: Navigation

    var body: some View {
        GeometryReader { geometry in
            HStack {
                Image("LogoIcon")
                    .resizable()
                    .aspectRatio(contentMode: .fit)

                Text("Mullvad VPN")
                Spacer()

                Button(action: openAccount) {
                    Image(systemName: "person")
                        .imageScale(.large)
                }.disabled(navigation.state == .loggedOut)

                Button(action: openSettings) {
                    Image(systemName: "gearshape")
                        .imageScale(.large)
                }
            }
        }
    }

    func openAccount() {
        navigation.openAccount()
    }

    func openSettings() {
        navigation.openSettings()
    }
}

#Preview {
    HeaderBar()
        .environment(Navigation())
}

