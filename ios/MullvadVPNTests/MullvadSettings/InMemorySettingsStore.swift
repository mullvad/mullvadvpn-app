//
//  InMemorySettingsStore.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-10-17.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadSettings

protocol Instantiable {
    init()
}

class InMemorySettingsStore<ThrownError: Error>: SettingsStore where ThrownError: Instantiable {
    private var settings = [SettingsKey: Data]()

    func read(key: SettingsKey) throws -> Data {
        guard settings.keys.contains(key), let value = settings[key] else { throw ThrownError() }
        return value
    }

    func write(_ data: Data, for key: SettingsKey) throws {
        settings[key] = data
    }

    func delete(key: SettingsKey) throws {
        settings.removeValue(forKey: key)
    }
}
