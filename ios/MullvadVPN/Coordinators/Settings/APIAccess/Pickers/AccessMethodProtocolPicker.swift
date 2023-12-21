//
//  AccessMethodProtocolPicker.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

/// Type implementing the access method protocol picker.
struct AccessMethodProtocolPicker {
    /// The navigation controller used for presenting the picker.
    let navigationController: UINavigationController

    /// Push access method protocol picker onto the navigation stack.
    /// - Parameters:
    ///   - currentValue: current selection.
    ///   - completion: a completion handler.
    func present(currentValue: AccessMethodKind, completion: @escaping (AccessMethodKind) -> Void) {
        let navigationController = navigationController

        let dataSource = AccessMethodProtocolPickerDataSource()
        let controller = ListItemPickerViewController(dataSource: dataSource, selectedItemID: currentValue)

        controller.navigationItem.title = NSLocalizedString(
            "SELECT_PROTOCOL_NAV_TITLE",
            tableName: "APIAccess",
            value: "Type",
            comment: ""
        )

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

        var id: AccessMethodKind { method }
        var text: String { method.localizedDescription }
    }

    let items: [Item] = AccessMethodKind.allUserDefinedKinds.map { Item(method: $0) }

    var itemCount: Int {
        items.count
    }

    func item(at indexPath: IndexPath) -> Item {
        items[indexPath.row]
    }

    func indexPath(for itemID: AccessMethodKind) -> IndexPath? {
        guard let index = items.firstIndex(where: { $0.id == itemID }) else { return nil }

        return IndexPath(row: index, section: 0)
    }
}
