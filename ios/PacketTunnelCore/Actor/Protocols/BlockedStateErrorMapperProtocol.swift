//
//  BlockedStateErrorMapperProtocol.swift
//  PacketTunnelCore
//
//  Created by pronebird on 14/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

/// A type responsible for mapping errors returned by dependencies of `PacketTunnelActor` to `BlockedStateReason`.
public protocol BlockedStateErrorMapperProtocol {
    func mapError(_ error: Error) -> BlockedStateReason
}
