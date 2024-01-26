//
//  ListAccessMethodInteractorProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings

/// Types describing API access list interactor.
protocol ListAccessMethodInteractorProtocol {
    /// Publisher that produces a list of method items upon persistent store modifications.
    var itemsPublisher: any Publisher<[ListAccessMethodItem], Never> { get }

    /// Publisher that produces the last reachable method item upon persistent store modifications.
    var itemInUsePublisher: any Publisher<ListAccessMethodItem?, Never> { get }

    /// Returns an item by id.
    func item(by id: UUID) -> ListAccessMethodItem?

    /// Fetch all items.
    func fetch() -> [ListAccessMethodItem]
}
