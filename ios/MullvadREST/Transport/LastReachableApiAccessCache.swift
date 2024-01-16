//
//  LastReachableApiAccessStorage.swift
//  MullvadREST
//
//  Created by Mojgan on 2024-01-08.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings
struct LastReachableApiAccessCache: Identifiable {
    /// `UserDefaults` key shared by both processes. Used to cache and synchronize last reachable api access method between them.
    private let key = "LastReachableConfigurationCacheKey"
    private var container: UserDefaults
    private let defaultValue: UUID

    init(defaultValue: UUID, container: UserDefaults) {
        self.container = container
        self.defaultValue = defaultValue
    }

    var id: UUID {
        get {
            guard let value = container.string(forKey: key) else {
                return defaultValue
            }
            return UUID(uuidString: value)!
        }
        set {
            container.set(newValue.uuidString, forKey: key)
        }
    }
}
