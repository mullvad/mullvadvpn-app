//
//  BlockedStateErrorMapperStub.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 18/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

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
