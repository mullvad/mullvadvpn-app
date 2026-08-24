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
import MullvadTypes

public struct CustomList: Codable, Equatable {
    public let id: UUID
    public var name: String
    public var locations: [RelayLocation]

    public init(id: UUID = UUID(), name: String, locations: [RelayLocation]) {
        self.id = id
        self.name = name
        self.locations = locations
    }
}
