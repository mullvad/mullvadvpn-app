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

struct RootView: View {
    @Environment(Navigation.self) private var navigation: Navigation
    private var viewModel: any LoggingIn

    init(viewModel: any LoggingIn) {
        self.viewModel = viewModel
    }

    // Need to create an account
    // Need to login into an account
    var body: some View {
        VStack {
            HeaderBar()
                .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: 200)
            Spacer()

            LoginView(viewModel: viewModel)
            CreateAccountPromptView()
                .background(Color.mullvadBackground)
                .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: 150)
        }
        .frame(minWidth: 0, maxWidth: .infinity, minHeight: 0, maxHeight: .infinity)
        .background(Color.mullvadBackground)
        .environment(navigation)
    }
}

// FIXME: Use MockProxyFactory.makeProxyFactory here instead ?
#Preview("Root View") {
    RootView(viewModel: LoginViewModelStub()).environment(Navigation())
}
