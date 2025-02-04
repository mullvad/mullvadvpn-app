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

struct CustomListsRepositoryStub: CustomListRepositoryProtocol {
    let customLists: [CustomList]

    func save(list: CustomList) throws {}

    func delete(id: UUID) {}

    func fetch(by id: UUID) -> CustomList? {
        nil
    }

    func fetchAll() -> [CustomList] {
        customLists
    }
}
