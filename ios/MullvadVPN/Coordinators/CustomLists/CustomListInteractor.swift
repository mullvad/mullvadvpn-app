//
//  CustomListInteractor.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

protocol CustomListInteractorProtocol {
    func fetchAllCustomLists() -> [CustomList]
    func saveCustomList(viewModel: CustomListViewModel) throws
    func deleteCustomList(id: UUID)
}

struct CustomListInteractor: CustomListInteractorProtocol {
    let repository: CustomListRepositoryProtocol

    func fetchAllCustomLists() -> [CustomList] {
        repository.fetchAll()
    }

    func saveCustomList(viewModel: CustomListViewModel) throws {
        try _ = repository.save(list: viewModel.customList)
    }

    func deleteCustomList(id: UUID) {
        repository.delete(id: id)
    }
}
