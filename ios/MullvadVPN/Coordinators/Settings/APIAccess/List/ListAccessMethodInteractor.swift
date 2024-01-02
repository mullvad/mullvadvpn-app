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
    let repo: AccessMethodRepositoryProtocol

    init(repo: AccessMethodRepositoryProtocol) {
        self.repo = repo
    }

    var publisher: any Publisher<[ListAccessMethodItem], Never> {
        repo.publisher.map { newElements in
            newElements.map { $0.toListItem() }
        }
    }

    func item(by id: UUID) -> ListAccessMethodItem? {
        repo.fetch(by: id)?.toListItem()
    }

    func fetch() -> [ListAccessMethodItem] {
        repo.fetchAll().map { $0.toListItem() }
    }
}

extension PersistentAccessMethod {
    func toListItem() -> ListAccessMethodItem {
        let sanitizedName = name.trimmingCharacters(in: .whitespaces)
        let itemName = sanitizedName.isEmpty ? kind.localizedDescription : sanitizedName
        let itemDetail = sanitizedName.isEmpty ? nil : kind.localizedDescription

        return ListAccessMethodItem(
            id: id,
            name: itemName,
            detail: isEnabled
                ? itemDetail
                : NSLocalizedString(
                    "LIST_ACCESS_METHODS_DISABLED",
                    tableName: "APIAccess",
                    value: "Disabled",
                    comment: ""
                )
        )
    }
}
