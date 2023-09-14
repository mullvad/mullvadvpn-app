//
//  DeviceCheck+.swift
//  PacketTunnel
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension DeviceCheck {
    var blockedStateReason: BlockedStateReason? {
        if case .invalid = accountVerdict {
            return .invalidAccount
        }

        if case .revoked = deviceVerdict {
            return .deviceRevoked
        }

        return nil
    }
}
