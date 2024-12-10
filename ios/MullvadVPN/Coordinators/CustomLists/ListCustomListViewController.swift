//
//  ListCustomListViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-03-06.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import MullvadSettings
import UIKit

private enum SectionIdentifier: Hashable {
    case `default`
}

private struct ItemIdentifier: Hashable {
    var id: UUID
}

private enum CellReuseIdentifier: String, CaseIterable, CellIdentifierProtocol {
    case `default`

    var cellClass: AnyClass {
        switch self {
        default: BasicCell.self
        }
    }
}

class ListCustomListViewController: UIViewController {
    private typealias DataSource = UITableViewDiffableDataSource<SectionIdentifier, ItemIdentifier>

    private let interactor: CustomListInteractorProtocol
    private var dataSource: DataSource?
    private var fetchedItems: [CustomList] = []
    private var tableView = UITableView(frame: .zero, style: .plain)
    var didSelectItem: ((CustomList) -> Void)?
    var didFinish: (() -> Void)?

    init(interactor: CustomListInteractorProtocol) {
        self.interactor = interactor
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor
        view.setAccessibilityIdentifier(.listCustomListsView)

        addSubviews()
        configureNavigationItem()
        configureDataSource()
        configureTableView()
    }

    func updateDataSource(reloadExisting: Bool, animated: Bool = true) {
        fetchedItems = interactor.fetchAll()
        var snapshot = NSDiffableDataSourceSnapshot<SectionIdentifier, ItemIdentifier>()
        snapshot.appendSections([.default])

        let itemIdentifiers = fetchedItems.map { ItemIdentifier(id: $0.id) }
        snapshot.appendItems(itemIdentifiers, toSection: .default)

        if reloadExisting {
            for item in fetchedItems {
                snapshot.reconfigureItems([ItemIdentifier(id: item.id)])
            }
        }

        dataSource?.apply(snapshot, animatingDifferences: animated)
    }

    private func addSubviews() {
        view.addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview()
        }
    }

    private func configureTableView() {
        tableView.delegate = self
        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero
        tableView.separatorStyle = .singleLine
        tableView.rowHeight = UIMetrics.SettingsCell.customListsCellHeight
        tableView.registerReusableViews(from: CellReuseIdentifier.self)
        tableView.setAccessibilityIdentifier(.listCustomListsTableView)
    }

    private func configureNavigationItem() {
        navigationItem.title = NSLocalizedString(
            "LIST_CUSTOM_LIST_NAVIGATION_TITLE",
            tableName: "CustomList",
            value: "Edit custom list",
            comment: ""
        )

        navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .done,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.didFinish?()
            })
        )

        navigationItem.rightBarButtonItem?.setAccessibilityIdentifier(.listCustomListDoneButton)
    }

    private func configureDataSource() {
        dataSource = DataSource(
            tableView: tableView,
            cellProvider: { [weak self] _, indexPath, itemIdentifier in
                self?.dequeueCell(at: indexPath, itemIdentifier: itemIdentifier)
            }
        )

        updateDataSource(reloadExisting: false, animated: false)
    }

    private func dequeueCell(
        at indexPath: IndexPath,
        itemIdentifier: ItemIdentifier
    ) -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: CellReuseIdentifier.default, for: indexPath)
        let item = fetchedItems[indexPath.row]
        var contentConfiguration = ListCellContentConfiguration()
        contentConfiguration.text = item.name
        cell.contentConfiguration = contentConfiguration

        if let cell = cell as? DynamicBackgroundConfiguration {
            cell.setAutoAdaptingBackgroundConfiguration(.mullvadListPlainCell(), selectionType: .dimmed)
        }

        if let cell = cell as? CustomCellDisclosureHandling {
            cell.disclosureType = .chevron
        }
        return cell
    }
}

extension ListCustomListViewController: UITableViewDelegate {
    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        tableView.deselectRow(at: indexPath, animated: false)

        let item = fetchedItems[indexPath.row]
        didSelectItem?(item)
    }
}
