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
import MullvadSettings
import MullvadTypes

/// The view model used by view controllers editing access method data.
struct AccessMethodViewModel: Identifiable {
    /// Socks configuration view model.
    struct Socks {
        /// Server IP address input.
        var server = ""
        /// Server port input.
        var port = ""
        /// Authentication username.
        var username = ""
        /// Authentication password.
        var password = ""
        /// Indicates whether authentication is enabled.
        var authenticate = false
    }

    /// Shadowsocks configuration view model.
    struct Shadowsocks {
        /// Server IP address input.
        var server = ""
        /// Server port input.
        var port = ""
        /// Server password.
        var password = ""
        /// Shadowsocks cipher.
        var cipher = ""
    }

    /// The unique identifier used for referencing the access method entry in a persistent store.
    var id = UUID()

    /// The user-defined name for access method.
    var name = ""

    /// The selected access method kind.
    /// Determines which subview model is used when presenting proxy configuration in UI.
    var method: AccessMethodKind = .shadowsocks

    /// The flag indicating whether configuration is enabled.
    var isEnabled = true

    /// Socks configuration view model.
    var socks = Socks()

    /// Shadowsocks configuration view model.
    var shadowsocks = Shadowsocks()
}
