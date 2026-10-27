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

struct AccountView: View {
    @Environment(Navigation.self) private var navigation: Navigation

    var body: some View {
        VStack(alignment: .leading) {
            Group {
                Text("Account")
                    .font(.largeTitle)
                Text("Device name")
                    .padding([.bottom])
                    .font(.title2)
                Text("account.deviceName")
                    .padding([.bottom])
                Text("Account number")
                    .font(.title2)
                Text("account.accountNumber")
                    .padding([.bottom])
                Text("Paid until")
                Text("account.paidUntil.formatted()")
            }

            Spacer()
            VStack(alignment: .leading) {
                Button("Log out") {
                    print("LOG OUT")
                    navigation.logOut()
                }.buttonStyle(DestructiveActionButton())
            }
        }
        .frame(
            maxWidth: .infinity,
            maxHeight: .infinity,
            alignment: .leading
        )
        .background(Color.mullvadBackground)
    }
}

#Preview {
    AccountView()
        .environment(Navigation())
}

struct ConfirmedActionButton: ButtonStyle {
    @Environment(\.isFocused) var focused: Bool
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.title2)
            .padding()
            .background(Color.mullvadSuccessColor)
            .foregroundStyle(.white)
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .scaleEffect(focused ? 1.1 : 1)
            .animation(.spring(), value: focused)
    }
}

struct DestructiveActionButton: ButtonStyle {
    @Environment(\.isFocused) var focused: Bool

    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .font(.title2)
            .padding()
            .background(Color.mullvadDangerColor)
            .foregroundStyle(.white)
            .clipShape(RoundedRectangle(cornerRadius: 10))
            .scaleEffect(focused ? 1.1 : 1)
            .animation(.spring(), value: focused)
    }
}
