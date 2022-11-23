//
//  MigrationFromUnversionedToV2.swift
//  MullvadVPN
//
//  Created by Sajad Vishkai on 2022-11-18.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes

class MigrationFromUnversionedToV2: Migration {
    func migrate(
        with store: SettingsStore,
        parser: SettingsParser,
        completion: @escaping (Error?) -> Void
    ) {
        do {
            let data = try store.read(key: .settings)

            let unversionedTunnelSettings = try parser.parseUnversionedPayload(
                as: TunnelSettingsV2.self,
                from: data
            )

            let settingsData = try parser.producePayload(
                unversionedTunnelSettings,
                version: SchemaVersion.v2.rawValue
            )

            try store.write(settingsData, for: .settings)

            completion(nil)
        } catch {
            completion(error)
        }
    }
}
