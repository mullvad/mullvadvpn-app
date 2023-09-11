//
//  Task+.swift
//  PacketTunnelCore
//
//  Created by pronebird on 11/09/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension Task where Success == Never, Failure == Never {
    static func sleep(seconds: UInt) async throws {
        try await sleep(nanoseconds: UInt64(seconds) * 1_000_000_000)
    }
}
