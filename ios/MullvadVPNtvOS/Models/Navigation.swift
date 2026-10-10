// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import SwiftUI

enum NavigationState {
    case loggedIn
    case loggedOut
}

enum FullScreenCover: Identifiable {
    var id: Self { self }
    case settings
    case account
}

@Observable
final class Navigation {
    var state: NavigationState = .loggedOut
    var fullScreenCover: FullScreenCover?

    func logIn() {
        state = .loggedIn
    }

    func logOut() {
        dismissFullScreenCover()
        state = .loggedOut
    }

    func openAccount() {
        fullScreenCover = .account
    }

    func openSettings() {
        fullScreenCover = .settings
    }

    func dismissFullScreenCover() {
        fullScreenCover = nil
    }
}
