//
//  AccessMethodRepositoryStub.swift
//  MullvadRESTTests
//
//  Created by Mojgan on 2024-01-02.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings

struct AccessMethodRepositoryStub: AccessMethodRepositoryDataSource {
    var directAccess: PersistentAccessMethod

    var accessMethodsPublisher: AnyPublisher<[PersistentAccessMethod], Never> {
        passthroughSubject.eraseToAnyPublisher()
    }

    let passthroughSubject: CurrentValueSubject<[PersistentAccessMethod], Never> = CurrentValueSubject([])

    init(accessMethods: [PersistentAccessMethod]) {
        directAccess = accessMethods.first(where: { $0.kind == .direct })!
        passthroughSubject.send(accessMethods)
    }

    func fetchAll() -> [PersistentAccessMethod] {
        passthroughSubject.value
    }

    func saveLastReachable(_ method: PersistentAccessMethod) {}

    func fetchLastReachable() -> PersistentAccessMethod {
        directAccess
    }
}
