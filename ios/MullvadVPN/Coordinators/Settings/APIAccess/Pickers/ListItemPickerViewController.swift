//
//  ListItemPickerViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 13/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// An item type used by list item data source.
protocol ListItemDataSourceItem<ID>: Identifiable {
    /// Item's text representation for UI presentation.
    var text: String { get }
}

/// A data source type used together with ``ListItemPickerViewController``.
protocol ListItemDataSourceProtocol<Item> {
    associatedtype Item: ListItemDataSourceItem

    /// Number of items in the data source.
    var itemCount: Int { get }

    /// Return item at index path.
    ///
    /// - Parameter indexPath: an index path.
    /// - Returns: the item corresponding to the given index path.
    func item(at indexPath: IndexPath) -> Item

    /// Get index path by item ID.
    ///
    /// - Parameter itemID: an item ID.
    /// - Returns: the index path that corresponds to the given item ID upon success, otherwise `nil`.
    func indexPath(for itemID: Item.ID) -> IndexPath?
}

/// A view controller presenting a list of items from which the user can choose one item.
class ListItemPickerViewController<DataSource: ListItemDataSourceProtocol>: UITableViewController {
    typealias Item = DataSource.Item

    private let dataSource: DataSource
    private var selectedItemID: Item.ID?
    private var scrolledToSelection = false

    var onSelect: ((Item) -> Void)?

    /// Designated initializer.
    /// - Parameters:
    ///   - dataSource: a data source.
    ///   - selectedValue: the initially selected item ID.
    init(dataSource: DataSource, selectedItemID: Item.ID?) {
        self.dataSource = dataSource
        self.selectedItemID = selectedItemID

        super.init(style: .plain)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor

        tableView.separatorInset = .zero
        tableView.separatorColor = .secondaryColor
        tableView.registerReusableViews(from: CellIdentifier.self)

        // Add extra inset to mimic built-in margin of a grouped table view. Without this the
        // transition between a plain and a grouped table view looks jarring.
        tableView.contentInset.top = UIMetrics.SettingsCell.apiAccessPickerListContentInsetTop
    }

    override func viewIsAppearing(_ animated: Bool) {
        super.viewIsAppearing(animated)

        guard !scrolledToSelection else { return }

        scrolledToSelection = true

        if let selectedItemID, let indexPath = dataSource.indexPath(for: selectedItemID) {
            tableView.scrollToRow(at: indexPath, at: .middle, animated: false)
        }
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let item = dataSource.item(at: indexPath)
        var configuration = UIListContentConfiguration.mullvadCell(tableStyle: .insetGrouped)
        configuration.text = item.text

        let cell = tableView.dequeueReusableView(withIdentifier: CellIdentifier.default, for: indexPath)
        cell.contentConfiguration = configuration

        if let cell = cell as? CustomCellDisclosureHandling {
            cell.disclosureType = item.id == selectedItemID ? .tick : .none
        }

        if let cell = cell as? DynamicBackgroundConfiguration {
            cell.setAutoAdaptingBackgroundConfiguration(.mullvadListPlainCell(), selectionType: .dimmed)
        }

        return cell
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        return dataSource.itemCount
    }

    override func tableView(_ tableView: UITableView, heightForRowAt indexPath: IndexPath) -> CGFloat {
        return UIMetrics.SettingsCell.apiAccessCellHeight
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let selectedItem = dataSource.item(at: indexPath)
        selectedItemID = selectedItem.id
        onSelect?(selectedItem)
    }
}

private enum CellIdentifier: String, CellIdentifierProtocol, CaseIterable {
    case `default`

    var cellClass: AnyClass {
        BasicCell.self
    }
}
