//
//  ListAccessMethodInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings

/// A concrete implementation of an API access list interactor.
struct ListAccessMethodInteractor: ListAccessMethodInteractorProtocol {
    let repository: AccessMethodRepositoryProtocol

    init(repository: AccessMethodRepositoryProtocol) {
        self.repository = repository
    }

    var itemsPublisher: AnyPublisher<[ListAccessMethodItem], Never> {
        repository.accessMethodsPublisher
            .map { methods in
                methods.map { $0.toListItem() }
            }
            .eraseToAnyPublisher()
    }

    var itemInUsePublisher: AnyPublisher<ListAccessMethodItem?, Never> {
        repository.lastReachableAccessMethodPublisher
            .map { $0.toListItem() }
            .eraseToAnyPublisher()
    }

    func item(by id: UUID) -> ListAccessMethodItem? {
        repository.fetch(by: id)?.toListItem()
    }

    func fetch() -> [ListAccessMethodItem] {
        repository.fetchAll().map { $0.toListItem() }
    }
}

extension PersistentAccessMethod {
    func toListItem() -> ListAccessMethodItem {
        let sanitizedName = name.trimmingCharacters(in: .whitespaces)
        let itemName = sanitizedName.isEmpty ? kind.localizedDescription : sanitizedName

        return ListAccessMethodItem(
            id: id,
            name: itemName,
            detail: isEnabled
                ? kind.localizedDescription
                : NSLocalizedString(
                    "LIST_ACCESS_METHODS_DISABLED",
                    tableName: "APIAccess",
                    value: "Disabled",
                    comment: ""
                ),
            isEnabled: isEnabled
        )
    }
}
