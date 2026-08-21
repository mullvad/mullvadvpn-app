// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

public struct TunnelSettingsManager: SettingsReaderProtocol {
    let settingsReader: SettingsReaderProtocol
    let onLoadSettingsHandler: ((Settings) -> Void)?

    public init(settingsReader: SettingsReaderProtocol, onLoadSettingsHandler: ((Settings) -> Void)? = nil) {
        self.settingsReader = settingsReader
        self.onLoadSettingsHandler = onLoadSettingsHandler
    }

    public func read() throws -> Settings {
        let settings = try settingsReader.read()
        onLoadSettingsHandler?(settings)
        return settings
    }
}
