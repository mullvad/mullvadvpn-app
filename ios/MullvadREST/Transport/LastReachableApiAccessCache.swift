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
    private var appStorage: AppStorage<String>

    init(key: String, defaultValue: UUID, container: UserDefaults) {
        self.appStorage = AppStorage(
            wrappedValue: defaultValue.uuidString,
            key: key,
            container: container
        )
    }

    var id: UUID {
        get {
            let value = appStorage.wrappedValue
            return UUID(uuidString: value)!
        }
        set {
            appStorage.wrappedValue = newValue.uuidString
        }
    }
}
