//
//  AccessMethodRepository.swift
//  MullvadVPN
//
//  Created by pronebird on 22/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation

class AccessMethodRepository: AccessMethodRepositoryProtocol {
    private var memoryStore: [PersistentAccessMethod] {
        didSet {
            publisher.send(memoryStore)
        }
    }

    let publisher: PassthroughSubject<[PersistentAccessMethod], Never> = .init()

    static let shared = AccessMethodRepository()

    init() {
        memoryStore = [
            PersistentAccessMethod(
                id: UUID(uuidString: "C9DB7457-2A55-42C3-A926-C07F82131994")!,
                name: "",
                isEnabled: true,
                proxyConfiguration: .direct
            ),
            PersistentAccessMethod(
                id: UUID(uuidString: "8586E75A-CA7B-4432-B70D-EE65F3F95084")!,
                name: "",
                isEnabled: true,
                proxyConfiguration: .bridges
            ),
        ]
    }

    func add(_ method: PersistentAccessMethod) {
        guard !memoryStore.contains(where: { $0.id == method.id }) else { return }

        memoryStore.append(method)
    }

    func update(_ method: PersistentAccessMethod) {
        guard let index = memoryStore.firstIndex(where: { $0.id == method.id }) else { return }

        memoryStore[index] = method
    }

    func delete(id: UUID) {
        guard let index = memoryStore.firstIndex(where: { $0.id == id }) else { return }

        // Prevent removing methods that have static UUIDs and are always present.
        let permanentMethod = memoryStore[index]
        if !permanentMethod.kind.isPermanent {
            memoryStore.remove(at: index)
        }
    }

    func fetch(by id: UUID) -> PersistentAccessMethod? {
        memoryStore.first { $0.id == id }
    }

    func fetchAll() -> [PersistentAccessMethod] {
        memoryStore
    }
}
