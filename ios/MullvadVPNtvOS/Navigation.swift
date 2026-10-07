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

@Observable
final class Navigation {
    let uniqueIdentifier = UUID().uuidString

    var state: NavigationState = .loggedOut
    var isPresentingSettings = false
    var isPresentingAccount = false

    func logIn() {
        state = .loggedIn
    }

    func logOut() {
        closeAccount()
        closeSettings()
        state = .loggedOut
    }

    func openAccount() {
        isPresentingAccount = true
    }

    func closeAccount() {
        isPresentingAccount = false
    }

    func openSettings() {
        isPresentingSettings = true
    }

    func closeSettings() {
        isPresentingSettings = false
    }
}
