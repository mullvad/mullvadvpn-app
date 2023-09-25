//
//  MockBlockedStateErrorMapper.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 18/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import PacketTunnelCore

class MockBlockedStateErrorMapper: BlockedStateErrorMapperProtocol {
    let block: (Error) -> BlockedStateReason

    init(block: @escaping (Error) -> BlockedStateReason) {
        self.block = block
    }

    func mapError(_ error: Error) -> BlockedStateReason {
        return block(error)
    }
}

extension MockBlockedStateErrorMapper {
    /// Returns a mock that maps all errors to `.unknown`.
    static func mock() -> MockBlockedStateErrorMapper {
        MockBlockedStateErrorMapper { _ in .unknown }
    }
}
