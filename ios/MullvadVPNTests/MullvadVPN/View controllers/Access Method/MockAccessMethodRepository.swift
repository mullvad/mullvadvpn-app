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
import MullvadMockData
import MullvadSettings
import MullvadTypes

final class MockAccessMethodRepository: AccessMethodRepositoryProtocol, @unchecked Sendable {
    private let stub: AccessMethodRepositoryStub

    private(set) var savedMethods: [(PersistentAccessMethod, Bool)] = []
    private(set) var deletedIDs: [UUID] = []
    private(set) var addDefaultsMethodsCalled = false

    var currentAccessMethodPublisher: AnyPublisher<PersistentAccessMethod, Never> {
        stub.accessMethodsPublisher
            .compactMap { $0.first(where: \.isEnabled) }
            .eraseToAnyPublisher()
    }

    init(accessMethods: [PersistentAccessMethod] = AccessMethodRepositoryStub.stub.fetchAll()) {
        stub = AccessMethodRepositoryStub(accessMethods: accessMethods)
    }

    func save(_ method: PersistentAccessMethod, notifyingAPI: Bool) {
        savedMethods.append((method, notifyingAPI))
    }

    func delete(id: UUID) {
        deletedIDs.append(id)
    }

    func fetch(by id: UUID) -> PersistentAccessMethod? {
        stub.fetchAll().first { $0.id == id }
    }

    func addDefaultsMethods() {
        addDefaultsMethodsCalled = true
    }

    // Forward AccessMethodRepositoryDataSource requirements.
    var shadowsocksCiphers: [String] { stub.shadowsocksCiphers }
    var directAccess: PersistentAccessMethod { stub.directAccess }
    var accessMethodsPublisher: AnyPublisher<[PersistentAccessMethod], Never> {
        stub.accessMethodsPublisher
    }

    func fetchAll() -> [PersistentAccessMethod] { stub.fetchAll() }

    func requestAccessMethod(_ method: PersistentAccessMethod) {
        stub.requestAccessMethod(method)
    }
    func fetchLastReachable() -> PersistentAccessMethod {
        stub.fetchLastReachable()
    }
    func infoHeaderConfig(for id: UUID) -> InfoHeaderConfig? {
        stub.infoHeaderConfig(for: id)
    }
}
