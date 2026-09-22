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

public actor InAppLogBuffer: Sendable {
    private var entries: [InAppLogEntry] = []

    public init() {}

    public func append(_ entry: InAppLogEntry) {
        entries.append(entry)
    }

    public func flush() -> [InAppLogEntry] {
        let result = entries
        entries.removeAll()

        return result
    }
}
