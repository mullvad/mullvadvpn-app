//
//  ShadowsocksCipherPicker.swift
//  MullvadVPN
//
//  Created by pronebird on 14/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import MullvadTypes
import UIKit

/// Type implementing the shadowsocks cipher picker.
@MainActor
struct ShadowsocksCipherPicker {
    /// The navigation controller used for presenting the picker.
    let navigationController: UINavigationController
    /// Supported Shadowsocks ciphers.
    let ciphers: [String]

    /// Push shadowsocks cipher picker onto the navigation stack.
    /// - Parameters:
    ///   - currentValue: current selection.
    ///   - completion: a completion handler.
    func present(currentValue: String, completion: @escaping (String) -> Void) {
        let navigationController = navigationController

        let dataSource = ShadowsocksCipherPickerDataSource(currentValue: currentValue, ciphers: ciphers)
        let controller = ListItemPickerViewController(dataSource: dataSource)

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
        let isEnabled: Bool

        var id: String { cipher }
        var text: String { cipher }
        var detailText: String? {
            isEnabled ? nil : NSLocalizedString("No longer supported", comment: "")
        }
    }

    let items: [Item]
    var selectedItem: Item?

    var itemCount: Int {
        items.count
    }

    init(currentValue: String, ciphers: [String]) {
        var items = ciphers.map { Item(cipher: $0, isEnabled: true) }
        let selectedItem = Item(cipher: currentValue, isEnabled: ciphers.contains(currentValue))

        if !items.contains(selectedItem) {
            items.append(selectedItem)
            items.sort { $0.id < $1.id }
        }

        self.items = items
        self.selectedItem = selectedItem
    }

    func item(at indexPath: IndexPath) -> Item {
        items[indexPath.row]
    }

    func indexPath(for item: Item) -> IndexPath? {
        guard let index = items.firstIndex(where: { $0.id == item.id }) else { return nil }
        return IndexPath(row: index, section: 0)
    }
}
