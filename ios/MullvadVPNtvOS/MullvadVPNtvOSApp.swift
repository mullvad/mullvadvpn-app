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
    init() {
        dependenciesTask = Task {
            await DependencyManager().start()
        }
    }

    // TODO: Async init of the dependencies in the splash screen
    // Then make sure we're using the correct thing when the first screen is shown
    // The dependencies have been loaded during the splash screen

    @State private var navigation = Navigation()
    @State private var loadedDepencencies = false
    @State var dependenciesTask: Task<DependencyManager, Never>
    @State var dependencies: DependencyManager?

    var body: some Scene {
        WindowGroup {
            Group {
                switch navigation.state {
                case .loggedIn:
                    MapView()
                case .loggedOut:
                    loadRootView()
                }
            }
            .task {
                _dependencies.wrappedValue = await dependenciesTask.value
                loadedDepencencies = true
            }
            .fullScreenCover(
                isPresented: $navigation.isPresentingAccount,
                content: {
                    AccountView()
                }
            )
            .environment(navigation)
        }
    }

    @ViewBuilder
    func loadRootView() -> some View {
        if loadedDepencencies {
            RootView(viewModel: dependencies!.makeLoginViewModel())
        } else {
            Text("Loading")
        }
    }
}
