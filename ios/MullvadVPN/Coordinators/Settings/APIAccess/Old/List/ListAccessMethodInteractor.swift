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

/// A concrete implementation of an API access list interactor.
struct ListAccessMethodInteractor: ListAccessMethodInteractorProtocol {
    let repository: AccessMethodRepositoryProtocol

    init(repository: AccessMethodRepositoryProtocol) {
        self.repository = repository
    }

    var itemsPublisher: AnyPublisher<[ListAccessMethodItem], Never> {
        repository.accessMethodsPublisher
            .receive(on: RunLoop.main)
            .map { methods in
                methods.map { $0.toListItem() }
            }
            .eraseToAnyPublisher()
    }

    var itemInUsePublisher: AnyPublisher<ListAccessMethodItem?, Never> {
        repository.currentAccessMethodPublisher
            .receive(on: RunLoop.main)
            .map { $0.toListItem() }
            .eraseToAnyPublisher()
    }

    var shadowsocksCiphers: [String] {
        repository.shadowsocksCiphers
    }

    func item(by id: UUID) -> ListAccessMethodItem? {
        accessMethod(by: id)?.toListItem()
    }

    func fetch() -> [ListAccessMethodItem] {
        repository.fetchAll().map { $0.toListItem() }
    }

    func accessMethod(by id: UUID) -> PersistentAccessMethod? {
        repository.fetch(by: id)
    }
}

extension PersistentAccessMethod {
    func toListItem() -> ListAccessMethodItem {
        let sanitizedName = name.trimmingCharacters(in: .whitespaces)
        let itemName = sanitizedName.isEmpty ? kind.localizedDescription : sanitizedName

        return ListAccessMethodItem(
            id: id,
            name: itemName,
            detail: kind.localizedDescription,
            isEnabled: isEnabled
        )
    }
}
