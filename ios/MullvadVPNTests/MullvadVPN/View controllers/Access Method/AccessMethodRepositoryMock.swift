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

final class AccessMethodRepositoryMock: AccessMethodRepositoryProtocol, @unchecked Sendable {

    // MARK: - Properties

    var shadowsocksCiphers: [String] = []

    var directAccess: PersistentAccessMethod

    private let accessMethodsSubject: CurrentValueSubject<[PersistentAccessMethod], Never>
    private let currentAccessMethodSubject: CurrentValueSubject<PersistentAccessMethod, Never>

    var accessMethodsPublisher: AnyPublisher<[PersistentAccessMethod], Never> {
        accessMethodsSubject.eraseToAnyPublisher()
    }

    var currentAccessMethodPublisher: AnyPublisher<PersistentAccessMethod, Never> {
        currentAccessMethodSubject.eraseToAnyPublisher()
    }

    private(set) var requestedAccessMethod: PersistentAccessMethod?
    private(set) var savedAccessMethod: PersistentAccessMethod?
    private(set) var savedNotifyingAPI: Bool?
    private(set) var deletedID: UUID?
    private(set) var fetchLastReachableCallCount = 0
    private(set) var addDefaultsMethodsCallCount = 0

    // MARK: - Initialization

    init(
        accessMethods: [PersistentAccessMethod] = [],
        directAccess: PersistentAccessMethod = .makePersistentAccessMethod()
    ) {
        self.directAccess = directAccess

        let initialMethod = accessMethods.first ?? directAccess

        self.accessMethodsSubject = CurrentValueSubject(accessMethods)
        self.currentAccessMethodSubject = CurrentValueSubject(initialMethod)
    }

    // MARK: - Data Source

    func fetchAll() -> [PersistentAccessMethod] {
        accessMethodsSubject.value
    }

    func requestAccessMethod(_ method: PersistentAccessMethod) {
        requestedAccessMethod = method
    }

    func fetchLastReachable() -> PersistentAccessMethod {
        fetchLastReachableCallCount += 1
        return currentAccessMethodSubject.value
    }

    // MARK: - Repository

    func save(_ method: PersistentAccessMethod, notifyingAPI: Bool) {
        savedAccessMethod = method
        savedNotifyingAPI = notifyingAPI

        var methods = accessMethodsSubject.value

        if let index = methods.firstIndex(where: { $0.id == method.id }) {
            methods[index] = method
        } else {
            methods.append(method)
        }

        accessMethodsSubject.send(methods)
    }

    func delete(id: UUID) {
        deletedID = id

        let methods = accessMethodsSubject.value.filter { $0.id != id }
        accessMethodsSubject.send(methods)
    }

    func fetch(by id: UUID) -> PersistentAccessMethod? {
        accessMethodsSubject.value.first { $0.id == id }
    }

    func addDefaultsMethods() {
        addDefaultsMethodsCallCount += 1
    }

    // MARK: - Test Helpers

    func setAccessMethods(_ methods: [PersistentAccessMethod]) {
        accessMethodsSubject.send(methods)
    }

    func setCurrentAccessMethod(_ method: PersistentAccessMethod) {
        currentAccessMethodSubject.send(method)
    }
}
