//
//  ListAccessMethodInteractorProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation

/// Types describing API access list interactor.
protocol ListAccessMethodInteractorProtocol {
    /// Returns an item by id.
    func item(by id: UUID) -> ListAccessMethodItem?

    /// Fetch all items.
    func fetch() -> [ListAccessMethodItem]

    var publisher: any Publisher<[ListAccessMethodItem], Never> { get }
}
