//
//  PostQuantumKeyExchangingUpdaterStub.swift
//  PacketTunnelCoreTests
//
//  Created by Mojgan on 2024-07-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
@testable import MullvadTypes
@testable import PacketTunnelCore

final class PostQuantumKeyExchangingUpdaterStub: PostQuantumKeyExchangingUpdaterProtocol {
    var reconfigurationHandler: ConfigUpdater?
}
