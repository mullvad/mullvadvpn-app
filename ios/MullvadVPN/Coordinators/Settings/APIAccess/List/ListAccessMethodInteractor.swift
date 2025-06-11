//
//  ListAccessMethodInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
        repository.lastReachableAccessMethodPublisher
            .receive(on: RunLoop.main)
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
        // the keys look like "ACCESS_METHOD_NAME:Mullvad bridges"
        let localizedName = Bundle.main.localizedString(
            forKey: "ACCESS_METHOD_NAME:\(sanitizedName)",
            value: sanitizedName,
            table: "APIAccess"
        )
        let itemName = localizedName.isEmpty ? kind.localizedDescription : localizedName

        return ListAccessMethodItem(
            id: id,
            name: itemName,
            detail: kind.localizedDescription,
            isEnabled: isEnabled
        )
    }
}
