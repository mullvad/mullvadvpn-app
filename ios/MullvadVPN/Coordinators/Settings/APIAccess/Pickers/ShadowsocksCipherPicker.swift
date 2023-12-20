//
//  ShadowsocksCipherPicker.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

/// Type implementing the shadowsocks cipher picker.
struct ShadowsocksCipherPicker {
    /// The navigation controller used for presenting the picker.
    let navigationController: UINavigationController

    /// Push shadowsocks cipher picker onto the navigation stack.
    /// - Parameters:
    ///   - currentValue: current selection.
    ///   - completion: a completion handler.
    func present(currentValue: ShadowsocksCipherOptions, completion: @escaping (ShadowsocksCipherOptions) -> Void) {
        let navigationController = navigationController

        let dataSource = ShadowsocksCipherPickerDataSource()
        let controller = ListItemPickerViewController(dataSource: dataSource, selectedItemID: currentValue)

        controller.navigationItem.title = NSLocalizedString(
            "SELECT_SHADOWSOCKS_CIPHER_NAV_TITLE",
            tableName: "APIAccess",
            value: "Cipher",
            comment: ""
        )

        controller.onSelect = { selectedItem in
            navigationController.popViewController(animated: true)
            completion(selectedItem.cipher)
        }

        navigationController.pushViewController(controller, animated: true)
    }
}

/// Type implementing the data source for the shadowsocks cipher picker.
struct ShadowsocksCipherPickerDataSource: ListItemDataSourceProtocol {
    struct Item: ListItemDataSourceItem {
        let cipher: ShadowsocksCipherOptions

        var id: ShadowsocksCipherOptions { cipher }
        var text: String { "\(cipher.rawValue.description)" }
    }

    let items = ShadowsocksCipherOptions.all.map { Item(cipher: $0) }

    var itemCount: Int {
        items.count
    }

    func item(at indexPath: IndexPath) -> Item {
        items[indexPath.row]
    }

    func indexPath(for itemID: ShadowsocksCipherOptions) -> IndexPath? {
        guard let index = items.firstIndex(where: { $0.id == itemID }) else { return nil }

        return IndexPath(row: index, section: 0)
    }
}
