// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadSettings
import MullvadTypes
import SwiftUI

@MainActor
protocol ListCustomListsViewModel {
    var customLists: [CustomList] { get }
    var customListInteractor: CustomListInteractorProtocol { get }
    var multihopContext: MultihopContext { get }
}

@Observable
class ListCustomListsViewModelImpl: ListCustomListsViewModel {
    var customLists: [CustomList] = []

    let customListInteractor: any CustomListInteractorProtocol
    let multihopContext: MultihopContext

    init(
        customListInteractor: any CustomListInteractorProtocol,
        multihopContext: MultihopContext
    ) {
        self.customListInteractor = customListInteractor
        self.multihopContext = multihopContext
        Task {
            self.customLists = customListInteractor.fetchAll()
        }
    }
}

class MockListCustomListsViewModel: ListCustomListsViewModel {
    let customLists: [CustomList] = [
        .init(name: "List 1", locations: []),
        .init(name: "List 2", locations: []),
    ]

    let customListInteractor: any CustomListInteractorProtocol = MockCustomListInteractor()

    let multihopContext: MultihopContext = .exit
}
