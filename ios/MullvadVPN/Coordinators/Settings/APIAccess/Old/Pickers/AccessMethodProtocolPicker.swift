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
import UIKit

/// Type implementing the access method protocol picker.
@MainActor
struct AccessMethodProtocolPicker {
    /// The navigation controller used for presenting the picker.
    let navigationController: UINavigationController

    /// Push access method protocol picker onto the navigation stack.
    /// - Parameters:
    ///   - currentValue: current selection.
    ///   - completion: a completion handler.
    func present(currentValue: AccessMethodKind, completion: @escaping (AccessMethodKind) -> Void) {
        let navigationController = navigationController

        let dataSource = AccessMethodProtocolPickerDataSource(currentValue: currentValue)
        let controller = ListItemPickerViewController(dataSource: dataSource)
        controller.view.setAccessibilityIdentifier(.accessMethodProtocolPickerView)

        controller.navigationItem.title = NSLocalizedString("Type", comment: "")

        controller.onSelect = { selectedItem in
            navigationController.popViewController(animated: true)
            completion(selectedItem.method)
        }

        navigationController.pushViewController(controller, animated: true)
    }
}

/// Type implementing the data source for the access method protocol picker.
struct AccessMethodProtocolPickerDataSource: ListItemDataSourceProtocol {
    struct Item: ListItemDataSourceItem {
        let method: AccessMethodKind

        var id: String { method.localizedDescription }
        var text: String { method.localizedDescription }
        var detailText: String? { nil }
        var isEnabled: Bool { true }
    }

    let items: [Item] = AccessMethodKind.allUserDefinedKinds.map { Item(method: $0) }
    var selectedItem: Item?

    var itemCount: Int {
        items.count
    }

    init(currentValue: AccessMethodKind) {
        self.selectedItem = Item(method: currentValue)
    }

    func item(at indexPath: IndexPath) -> Item {
        items[indexPath.row]
    }

    func indexPath(for item: Item) -> IndexPath? {
        guard let index = items.firstIndex(where: { $0.id == item.id }) else { return nil }
        return IndexPath(row: index, section: 0)
    }
}
