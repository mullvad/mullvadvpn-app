//
//  ShadowsocksCacheCleaner.swift
//  MullvadREST
//
//  Created by Marco Nikic on 2025-09-18.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
import MullvadTypes

public class ShadowsocksCacheCleaner: MullvadAccessMethodChangeListening {
    let cache: ShadowsocksConfigurationCacheProtocol
    var lastChangedUUID = UUID(uuidString: "00000000-0000-0000-0000-000000000000")!

    public init(cache: ShadowsocksConfigurationCacheProtocol) {
        self.cache = cache
    }

    public func accessMethodChangedTo(_ uuid: UUID) {
        if lastChangedUUID == AccessMethodRepository.bridgeId {
            try? cache.clear()
        }
        lastChangedUUID = uuid
    }
}
