//
//  RelaySelectorProtocol.swift
//  PacketTunnel
//
//  Created by pronebird on 08/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes
import RelaySelector

public protocol RelaySelectorProtocol {
    func selectRelay(with constraints: RelayConstraints, connectionAttemptFailureCount: UInt) throws
        -> RelaySelectorResult
}
