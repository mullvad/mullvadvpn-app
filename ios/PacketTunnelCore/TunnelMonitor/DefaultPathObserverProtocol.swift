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
import Network

/// A type providing default path access and observation.
public protocol DefaultPathObserverProtocol: Sendable {
    /// Returns current default path or `nil` if unknown yet.
    var currentPathStatus: Network.NWPath.Status { get }

    /// Start observing changes to `defaultPath`.
    /// This call must be idempotent. Multiple calls to start should replace the existing handler block.
    func start(_ body: @escaping @Sendable (Network.NWPath.Status) -> Void)

    /// Stop observing changes to `defaultPath`.
    func stop()
}
