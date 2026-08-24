// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Combine
import MullvadSettings
import MullvadTypes

class CustomListsRepositoryStub: CustomListRepositoryProtocol {
    var customLists: [CustomList]

    init(customLists: [CustomList] = []) {
        self.customLists = customLists
    }

    func save(list: CustomList) throws {
        delete(id: list.id)
        customLists.append(list)
    }

    func delete(id: UUID) {
        customLists.removeAll { $0.id == id }
    }

    func fetch(by id: UUID) -> CustomList? {
        customLists.first { $0.id == id }
    }

    func fetchAll() -> [CustomList] {
        customLists
    }
}
