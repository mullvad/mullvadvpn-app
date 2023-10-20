//
//  DeviceCheck+BlockedStateReason.swift
//  PacketTunnel
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import PacketTunnelCore

extension DeviceCheck {
    /// Returns blocked state reason inferred from the device check result.
    var blockedStateReason: BlockedStateReason? {
        if case .invalid = accountVerdict {
            return .invalidAccount
        }

        if case .revoked = deviceVerdict {
            return .deviceRevoked
        }

        if case .expired = accountVerdict {
            return .accountExpired
        }

        return nil
    }
}
