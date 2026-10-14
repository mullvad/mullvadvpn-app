// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings

/// The type implementing the interface for persisting changes to the underlying access method view model in the editing context.
protocol EditAccessMethodInteractorProtocol: ProxyConfigurationInteractorProtocol {
    /// Available Shadowsocks ciphers.
    var shadowsocksCiphers: [String] { get }

    /// Whether the view should show a breadcrumb or not.
    var shouldShowBreadcrumb: Bool { get }

    /// Save changes to persistent store.
    ///
    /// - Calling this method when the underlying view model fails validation does nothing.
    /// - View controllers are responsible to validate the view model before calling this method.
    func saveAccessMethod()

    /// Delete the access method from persistent store.
    ///
    /// - Calling this method multiple times does nothing.
    /// - View model does not have to pass validation for this method to work as the identifier field is the only requirement.
    /// - View controller presenting the UI for editing the access method must be dismissed after calling this method.
    func deleteAccessMethod()
}
