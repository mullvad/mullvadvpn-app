//
//  InMemorySettingsStore.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol Instantiable {
    init()
}

class InMemorySettingsStore<ThrownError: Error>: SettingsStore, @unchecked Sendable where ThrownError: Instantiable {
    private var settings = [SettingsKey: Data]()
    let queue = DispatchQueue(label: "com.mullvad.vpn.tests.inMemorySettingsStore")

    func read(key: SettingsKey) throws -> Data {
        try queue.sync {
            guard let value = settings[key] else { throw ThrownError() }
            return value
        }
    }

    func write(_ data: Data, for key: SettingsKey) throws {
        queue.sync {
            self.settings[key] = data
        }
    }

    func delete(key: SettingsKey) throws {
        queue.sync {
            _ = self.settings.removeValue(forKey: key)
        }
    }

    func reset() {
        queue.sync {
            self.settings.removeAll()
        }
    }
}
