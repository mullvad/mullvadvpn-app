//
//  ShadowsocksCipherPicker.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadRustRuntime
import MullvadSettings
import MullvadTypes
import UIKit

/// Type implementing the shadowsocks cipher picker.
@MainActor
struct ShadowsocksCipherPicker {
    /// The navigation controller used for presenting the picker.
    let navigationController: UINavigationController

    /// Push shadowsocks cipher picker onto the navigation stack.
    /// - Parameters:
    ///   - currentValue: current selection.
    ///   - completion: a completion handler.
    func present(currentValue: String, completion: @escaping (String) -> Void) {
        let navigationController = navigationController

        let dataSource = ShadowsocksCipherPickerDataSource()
        let currentItem = ShadowsocksCipherPickerDataSource.Item(cipher: currentValue)
        let controller = ListItemPickerViewController(dataSource: dataSource, selectedItem: currentItem)

        controller.navigationItem.title = NSLocalizedString("Cipher", comment: "")

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
        let cipher: String

        var id: String { cipher }
        var text: String { cipher }
    }

    let items = ShadowsocksCipherProvider.getCiphers()
        .sorted()
        .map { Item(cipher: $0) }

    var itemCount: Int {
        items.count
    }

    func item(at indexPath: IndexPath) -> Item {
        items[indexPath.row]
    }

    func indexPath(for item: Item) -> IndexPath? {
        guard let index = items.firstIndex(where: { $0.id == item.id }) else { return nil }

        return IndexPath(row: index, section: 0)
    }
}
