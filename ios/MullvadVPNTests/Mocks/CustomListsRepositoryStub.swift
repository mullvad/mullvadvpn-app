//
//  MockCustomListsRepository.swift
//  MullvadVPNTests
//
//  Created by Jon Petersson on 2024-02-29.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import MullvadTypes

struct CustomListsRepositoryStub: CustomListRepositoryProtocol {
    let customLists: [CustomList]

    var publisher: AnyPublisher<[CustomList], Never> {
        PassthroughSubject().eraseToAnyPublisher()
    }

    init(customLists: [CustomList]) {
        self.customLists = customLists
    }

    func update(_ list: CustomList) {}

    func delete(id: UUID) {}

    func fetch(by id: UUID) -> CustomList? {
        nil
    }

    func create(_ name: String, locations: [RelayLocation]) throws -> CustomList {
        CustomList(name: "", locations: [])
    }

    func fetchAll() -> [CustomList] {
        customLists
    }
}
