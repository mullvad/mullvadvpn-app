//
//  CustomListInteractor.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-15.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

protocol CustomListInteractorProtocol {
    func fetchAll() -> [CustomList]
    func save(viewModel: CustomListViewModel) throws
    func delete(id: UUID)
}

struct CustomListInteractor: CustomListInteractorProtocol {
    let repository: CustomListRepositoryProtocol

    func fetchAll() -> [CustomList] {
        repository.fetchAll()
    }

    func save(viewModel: CustomListViewModel) throws {
        try repository.save(list: viewModel.customList)
    }

    func delete(id: UUID) {
        repository.delete(id: id)
    }
}
