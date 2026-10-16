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

enum AccessMethodRepositoryError : Error {
    case notFound
}

protocol EditAccessMethodInteractorProtocol {
    func delete(_ method: PersistentAccessMethod)
    func save(_ method: PersistentAccessMethod)
    func canBeDisabled() -> Bool
    func fetch(_ id: UUID) throws -> PersistentAccessMethod
}

final class EditAccessMethodInteractor: EditAccessMethodInteractorProtocol {
    private let repository: AccessMethodRepositoryProtocol
    private var list: [PersistentAccessMethod] = []
    private var cancellables = Set<Combine.AnyCancellable>()

    init(repository: AccessMethodRepositoryProtocol) {
        self.repository = repository
        self.list = repository.fetchAll()
        repository.accessMethodsPublisher
            .receive(on: DispatchQueue.main)
            .sink { [weak self] updatedList in
                self?.list = updatedList
            }
            .store(in: &cancellables)
    }

    func save(_ method: PersistentAccessMethod) {
        repository.save(method, notifyingAPI: true)
    }

    func canBeDisabled() -> Bool {
        list.filter({ $0.isEnabled }).count > 1
    }

    func fetch(_ id: UUID) throws -> PersistentAccessMethod {
        guard let accessMethod = list.first(where: { $0.id == id }) else {
            throw AccessMethodRepositoryError.notFound
        }

        return accessMethod
    }

    func delete(_ method: PersistentAccessMethod) {
        repository.delete(id: method.id)
    }
}
