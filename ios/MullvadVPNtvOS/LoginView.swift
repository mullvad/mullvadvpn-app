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

struct LoginView: View {

    @State private var accountNumber: String = ""
    @State private var accountTextColor = Palette.white60
    @Environment(Navigation.self) private var navigation: Navigation

    var body: some View {
        VStack(alignment: .leading) {
            Text("Login")
                .bold()
                .font(.title)
            Text("Enter your account number")
                .font(.headline)
                .foregroundStyle(accountTextColor)
            Form {
                TextField("Account", text: $accountNumber,
                          prompt: Text("0000 0000 0000 0000")
                    .foregroundStyle(.gray)
                )
            }
            .onSubmit {
                login()
            }
        }
    }

    func login() {
        print("Logging in with \(accountNumber)")
        if validateAccountNumber() {
            accountTextColor = Palette.white60
            navigation.logIn()
        } else {
            accountTextColor = Palette.dangerColor
        }
    }

    func validateAccountNumber() -> Bool {
        accountNumber.allSatisfy( { $0.isNumber })
    }
}

#Preview {
    LoginView().environment(Navigation())
}
