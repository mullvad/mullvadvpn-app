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

@main
struct MullvadVPNtvOSApp: App {
    @State private var navigation = Navigation()
    @State var dependencies: DependencyManager?

    var body: some Scene {
        WindowGroup {
            Group {
                if let dependencies {
                    switch navigation.state {
                    case .loggedIn:
                        MapView()
                    case .loggedOut:
                        RootView(viewModel: dependencies.makeLoginViewModel())
                    }
                } else {
                    Text("Loading")
                }
            }
            .task {
                if dependencies == nil {
                    dependencies = await DependencyManager.live()
                }
            }
            .fullScreenCover(item: $navigation.fullScreenCover) { item in
                switch item {
                case .account:
                    AccountView()
                case .settings:
                    EmptyView()
                }
            }
            .environment(navigation)
            .environment(dependencies)
        }
    }
}
