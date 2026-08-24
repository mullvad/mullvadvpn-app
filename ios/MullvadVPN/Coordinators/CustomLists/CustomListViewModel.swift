// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes

struct CustomListViewModel {
    var id: UUID
    var name: String
    var locations: [RelayLocation]
    let tableSections: [CustomListSectionIdentifier]

    var customList: CustomList {
        CustomList(id: id, name: name, locations: locations)
    }

    mutating func update(with list: CustomList) {
        name = list.name
        locations = list.locations
    }
}
