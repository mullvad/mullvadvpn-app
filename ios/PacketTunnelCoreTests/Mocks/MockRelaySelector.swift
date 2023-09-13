//
//  MockRelaySelector.swift
//  PacketTunnelCoreTests
//
//  Created by pronebird on 05/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import PacketTunnelCore
import RelaySelector

struct MockRelaySelector: RelaySelectorProtocol {
    let block: (RelayConstraints, UInt) throws -> RelaySelectorResult

    func selectRelay(
        with constraints: RelayConstraints,
        connectionAttemptFailureCount: UInt
    ) throws -> RelaySelectorResult {
        return try block(constraints, connectionAttemptFailureCount)
    }
}
