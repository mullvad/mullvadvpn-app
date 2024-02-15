//
//  CustomListInteractor.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings

protocol CustomListInteractorProtocol {
    func createCustomList(viewModel: CustomListViewModel) throws
    func updateCustomList(viewModel: CustomListViewModel)
    func deleteCustomList(id: UUID)
}

struct CustomListInteractor: CustomListInteractorProtocol {
    let repository: CustomListRepositoryProtocol

    func createCustomList(viewModel: CustomListViewModel) throws {
        try _ = repository.create(viewModel.name, locations: viewModel.locations)
    }

    func updateCustomList(viewModel: CustomListViewModel) {
        repository.update(viewModel.customList)
    }

    func deleteCustomList(id: UUID) {
        repository.delete(id: id)
    }
}
