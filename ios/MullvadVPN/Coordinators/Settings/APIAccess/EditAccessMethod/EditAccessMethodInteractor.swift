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

enum EditAccessMethodError: Error {
    case notFound
    case cannotDisableLastEnabledMethod
}

protocol EditAccessMethodInteractorProtocol {
    func delete(_ method: PersistentAccessMethod)
    func save(_ method: PersistentAccessMethod)
    func setEnabled(
        _ isEnabled: Bool,
        for method: PersistentAccessMethod
    ) -> Result<PersistentAccessMethod, EditAccessMethodError>
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
            .sink { [weak self] updatedList in
                self?.list = updatedList
            }
            .store(in: &cancellables)
    }

    func save(_ method: PersistentAccessMethod) {
        repository.save(method, notifyingAPI: true)
    }

    func setEnabled(
        _ isEnabled: Bool,
        for method: PersistentAccessMethod
    ) -> Result<PersistentAccessMethod, EditAccessMethodError> {
        if !isEnabled && list.filter(\.isEnabled).count <= 1 {
            return .failure(.cannotDisableLastEnabledMethod)
        }

        var updatedMethod = method
        updatedMethod.isEnabled = isEnabled
        save(updatedMethod)

        return .success(updatedMethod)
    }

    func fetch(_ id: UUID) throws -> PersistentAccessMethod {
        guard let accessMethod = list.first(where: { $0.id == id }) else {
            throw EditAccessMethodError.notFound
        }

        return accessMethod
    }

    func delete(_ method: PersistentAccessMethod) {
        repository.delete(id: method.id)

        // Enable direct access if all methods are disabled
        if list.isEmpty {
            save(repository.directAccess)
        }
    }
}
