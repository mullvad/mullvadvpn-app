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
    let reporepository: AccessMethodRepositoryProtocol

    init(repository: AccessMethodRepositoryProtocol) {
        self.reporepository = repository
    }

    var publisher: any Publisher<[ListAccessMethodItem], Never> {
        reporepository.publisher.map { newElements in
            newElements.map { $0.toListItem() }
        }
    }

    func item(by id: UUID) -> ListAccessMethodItem? {
        reporepository.fetch(by: id)?.toListItem()
    }

    func fetch() -> [ListAccessMethodItem] {
        reporepository.fetchAll().map { $0.toListItem() }
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
                )
        )
    }
}
