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

struct CreateAccountPromptView: View {
    @Environment(Navigation.self) private var navigation: Navigation

    var body: some View {
        GeometryReader { geometry in
            VStack {
                Spacer()
                Text("Don't have an account number?")
                Button("Create an account", action: createAccount)
            }.frame(width: geometry.size.width)
        }
    }

    func createAccount() {
        print("Creating an account")
    }
}

#Preview {
    CreateAccountPromptView()
        .environment(Navigation())
}
