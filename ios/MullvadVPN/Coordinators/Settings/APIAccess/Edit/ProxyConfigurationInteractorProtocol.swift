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
import MullvadTypes

/// The type implementing the facilities for testing proxy configuration.
@MainActor
protocol ProxyConfigurationInteractorProtocol: Sendable {
    /// Start testing proxy configuration with data from view model.
    ///
    /// - It's expected that the completion handler is not called if testing is cancelled.
    /// - The interactor should update the underlying view model to indicate the progress of testing. The view controller is expected to keep track of that and update
    ///   the UI accordingly.
    ///
    /// - Parameter completion: completion handler receiving `true` if the test succeeded, otherwise `false`.
    func startProxyConfigurationTest(_ completion: (@Sendable (Bool) -> Void)?)

    /// Cancel currently running configuration test.
    /// The interactor is expected to reset the testing status to the initial.
    func cancelProxyConfigurationTest()
}

extension ProxyConfigurationInteractorProtocol {
    /// Start testing proxy configuration with data from view model.
    func startProxyConfigurationTest() {
        startProxyConfigurationTest(nil)
    }
}
