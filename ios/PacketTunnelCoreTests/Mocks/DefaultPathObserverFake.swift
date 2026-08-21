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
import NetworkExtension
import PacketTunnelCore

/// Default path observer fake that uses in-memory storage to keep current path and provides a method to simulate path change from tests.
class DefaultPathObserverFake: DefaultPathObserverProtocol, @unchecked Sendable {
    var currentPathStatus: Network.NWPath.Status { .satisfied }
    private var defaultPathHandler: ((Network.NWPath.Status) -> Void)?

    public var onStart: (() -> Void)?
    public var onStop: (() -> Void)?

    func start(_ body: @escaping (Network.NWPath.Status) -> Void) {
        defaultPathHandler = body
        onStart?()
    }

    func stop() {
        defaultPathHandler = nil
        onStop?()
    }

    /// Simulate network path update.
    func updatePath(_ newPath: Network.NWPath.Status) {}
}
