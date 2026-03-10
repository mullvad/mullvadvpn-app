//
//  ListItemPickerViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 13/11/2023.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// An item type used by list item data source.
protocol ListItemDataSourceItem: Equatable {
    var id: String { get }
    var text: String { get }
    var detailText: String? { get }
    var isEnabled: Bool { get }
}

/// A data source type used together with ``ListItemPickerViewController``.
protocol ListItemDataSourceProtocol<Item> {
    associatedtype Item: ListItemDataSourceItem

    /// Number of items in the data source.
    var itemCount: Int { get }

    /// The currently selected item.
    var selectedItem: Item? { get set }

    /// Return item at index path.
    ///
    /// - Parameter indexPath: an index path.
    /// - Returns: the item corresponding to the given index path.
    func item(at indexPath: IndexPath) -> Item

    /// Get index path by item ID.
    ///
    /// - Parameter cipher: the item ID.
    /// - Returns: the index path that corresponds to the given ID upon success, otherwise `nil`.
    func indexPath(for item: Item) -> IndexPath?
}

/// A view controller presenting a list of items from which the user can choose one item.
class ListItemPickerViewController<DataSource: ListItemDataSourceProtocol>: UITableViewController {
    typealias Item = DataSource.Item

    private var dataSource: DataSource
    private var scrolledToSelection = false

    var onSelect: ((Item) -> Void)?

    /// Designated initializer.
    /// - Parameters:
    ///   - dataSource: a data source.
    init(dataSource: DataSource) {
        self.dataSource = dataSource
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

        if let selectedItem = dataSource.selectedItem, let indexPath = dataSource.indexPath(for: selectedItem) {
            tableView.scrollToRow(at: indexPath, at: .middle, animated: false)
        }
    }

    override func tableView(_ tableView: UITableView, cellForRowAt indexPath: IndexPath) -> UITableViewCell {
        let item = dataSource.item(at: indexPath)

        var configuration = ListCellContentConfiguration()
        configuration.text = item.text
        configuration.tertiaryText = item.detailText
        configuration.isEnabled = item.isEnabled
        configuration.isSelected = item == dataSource.selectedItem

        let cell = tableView.dequeueReusableView(withIdentifier: CellIdentifier.default, for: indexPath)
        cell.contentConfiguration = configuration

        if let cell = cell as? DynamicBackgroundConfiguration {
            cell.isUserInteractionEnabled = item.isEnabled
            cell.setAutoAdaptingBackgroundConfiguration(.mullvadListPlainCell(), selectionType: .dimmed)
        }

        return cell
    }

    override func tableView(_ tableView: UITableView, numberOfRowsInSection section: Int) -> Int {
        dataSource.itemCount
    }

    override func tableView(_ tableView: UITableView, heightForRowAt indexPath: IndexPath) -> CGFloat {
        UITableView.automaticDimension
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        tableView.deselectRow(at: indexPath, animated: false)

        let item = dataSource.item(at: indexPath)
        guard item.isEnabled else {
            return
        }

        dataSource.selectedItem = item
        onSelect?(item)
    }
}

private enum CellIdentifier: String, CellIdentifierProtocol, CaseIterable {
    case `default`

    var cellClass: AnyClass {
        BasicCell.self
    }
}
