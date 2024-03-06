//
//  InMemoryCustomListRepository.swift
//  MullvadVPN
//
//  Created by Mojgan on 2024-01-31.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import MullvadSettings
import MullvadTypes

class InMemoryCustomListRepository: CustomListRepositoryProtocol {
    private var customRelayLists: [CustomList] = [
        CustomList(
            id: UUID(uuidString: "F17948CB-18E2-4F84-82CD-5780F94216DB")!,
            name: "Netflix",
            locations: [.city("al", "tia")]
        ),
        CustomList(
            id: UUID(uuidString: "4104C603-B35D-4A64-8865-96C0BF33D57F")!,
            name: "Streaming",
            locations: [
                .city("us", "dal"),
                .country("se"),
                .city("de", "ber"),
            ]
        ),
    ]

    func save(list: MullvadSettings.CustomList) throws {
        if let index = customRelayLists.firstIndex(where: { $0.id == list.id }) {
            customRelayLists[index] = list
        } else if customRelayLists.contains(where: { $0.name == list.name }) {
            throw CustomRelayListError.duplicateName
        } else {
            customRelayLists.append(list)
        }
    }

    func delete(id: UUID) {
        if let index = customRelayLists.firstIndex(where: { $0.id == id }) {
            customRelayLists.remove(at: index)
        }
    }

    func fetch(by id: UUID) -> CustomList? {
        return customRelayLists.first(where: { $0.id == id })
    }

    func fetchAll() -> [CustomList] {
        customRelayLists
    }
}
