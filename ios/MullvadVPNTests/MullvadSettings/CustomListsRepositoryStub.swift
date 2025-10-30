//
//  MockCustomListsRepository.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
