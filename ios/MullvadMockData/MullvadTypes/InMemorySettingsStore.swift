// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadSettings

public protocol Instantiable {
    init()
}

public class InMemorySettingsStore<ThrownError: Error>: SettingsStore, @unchecked Sendable
where ThrownError: Instantiable {
    private var settings = [SettingsKey: Data]()
    let queue = DispatchQueue(label: "com.mullvad.vpn.tests.inMemorySettingsStore")

    public init(settings: [SettingsKey: Data] = [:]) {
        self.settings = settings
    }

    public func read(key: SettingsKey) throws -> Data {
        try queue.sync {
            guard let value = settings[key] else { throw ThrownError() }
            return value
        }
    }

    public func write(_ data: Data, for key: SettingsKey) throws {
        queue.sync {
            self.settings[key] = data
        }
    }

    public func delete(key: SettingsKey) throws {
        queue.sync {
            _ = self.settings.removeValue(forKey: key)
        }
    }

    public func reset() {
        queue.sync {
            self.settings.removeAll()
        }
    }
}

public struct SettingNotFound: Error, Instantiable {
    public init() {}
}
