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

/// Type implementing access method proxy configuration testing.
protocol ProxyConfigurationTesterProtocol {
    /// Start testing proxy configuration.
    /// - Parameters:
    ///   - configuration: a proxy configuration.
    ///   - completion: a completion handler that receives `nil` upon success, otherwise the underlying error.
    func start(configuration: PersistentAccessMethod, completion: @escaping @Sendable (Error?) -> Void)

    /// Cancel testing proxy configuration.
    func cancel()
}
