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
import PacketTunnelCore

/// Blocked state error mapper stub that can be configured with a block to simulate a desired behavior.
class BlockedStateErrorMapperStub: BlockedStateErrorMapperProtocol {
    let block: (Error) -> BlockedStateReason

    /// Initialize a stub that always returns .unknown block reason.
    init() {
        self.block = { _ in .unknown }
    }

    /// Initialize a stub with custom error mapper block.
    init(block: @escaping (Error) -> BlockedStateReason) {
        self.block = block
    }

    func mapError(_ error: Error) -> BlockedStateReason {
        return block(error)
    }
}
