//
//  AccessMethodRepositoryStub.swift
//  MullvadRESTTests
//
//  Created by Mojgan on 2024-01-02.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings

typealias PersistentAccessMethod = MullvadSettings.PersistentAccessMethod
class AccessMethodRepositoryStub: AccessMethodRepositoryDataSource {
    var directAccess: MullvadSettings.PersistentAccessMethod

    var publisher: AnyPublisher<[MullvadSettings.PersistentAccessMethod], Never> {
        passthroughSubject.eraseToAnyPublisher()
    }

    let passthroughSubject: CurrentValueSubject<[PersistentAccessMethod], Never> = CurrentValueSubject([])

    init(accessMethods: [PersistentAccessMethod]) {
        directAccess = accessMethods.first(where: { $0.kind == .direct })!
        passthroughSubject.send(accessMethods)
    }
}
