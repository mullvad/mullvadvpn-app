//
//  UserDefaults+Extensions.swift
//  TunnelManager
//
//  Created by Marco Nikic on 2023-07-24.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadTypes

extension UserDefaults: AttemptsRecording {
    public func record(_ attempts: Int) {
        set(attempts, forKey: ApplicationConfiguration.connectionAttemptsSharedCacheKey)
    }
}
