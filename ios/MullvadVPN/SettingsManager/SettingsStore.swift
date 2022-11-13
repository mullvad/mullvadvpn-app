//
//  SettingsStore.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-22.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

protocol SettingsStore {
    func read(key: SettingsKey) throws -> Data
    func write(_ data: Data, for key: SettingsKey) throws
    func delete(key: SettingsKey) throws
}
