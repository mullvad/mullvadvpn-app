//
//  ListAccessMethodViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

enum ListAccessMethodSectionIdentifier: Hashable {
    case primary
}

struct ListAccessMethodItemIdentifier: Hashable {
    var id: UUID
}

/// View controller presenting a list of API access methods.
class ListAccessMethodViewController: UIViewController, UITableViewDelegate {
    private let headerView = ListAccessMethodHeaderView()
    private let interactor: ListAccessMethodInteractorProtocol
    private var cancellables = Set<AnyCancellable>()

    private var dataSource: UITableViewDiffableDataSource<
        ListAccessMethodSectionIdentifier,
        ListAccessMethodItemIdentifier
    >?
    private var fetchedItems: [ListAccessMethodItem] = []
    private let contentController = UITableViewController(style: .plain)
    private var tableView: UITableView {
        contentController.tableView
    }

    weak var delegate: ListAccessMethodViewControllerDelegate?

    /// Designated initializer.
    /// - Parameter interactor: the object implementing access and manipulation of the API access list.
    init(interactor: ListAccessMethodInteractorProtocol) {
        self.interactor = interactor
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        headerView.onAbout = { [weak self] in
            self?.sendAbout()
        }

        view.backgroundColor = .secondaryColor

        tableView.delegate = self
        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset = .zero

        tableView.registerReusableViews(from: CellReuseIdentifier.self)

        view.addConstrainedSubviews([headerView, tableView]) {
            headerView.pinEdgesToSuperview(.all().excluding(.bottom))
            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: headerView.bottomAnchor)
        }

        addChild(contentController)
        contentController.didMove(toParent: self)

        interactor.publisher.sink { _ in
            self.updateDataSource(animated: true)
        }
        .store(in: &cancellables)

        configureNavigationItem()
        configureDataSource()
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        let container = UIView()

        let button = AppButton(style: .tableInsetGroupedDefault)
        button.setTitle(
            NSLocalizedString(
                "LIST_ACCESS_METHODS_ADD_BUTTON",
                tableName: "APIAccess",
                value: "Add",
                comment: ""
            ),
            for: .normal
        )
        button.addAction(UIAction { [weak self] _ in
            self?.sendAddNew()
        }, for: .touchUpInside)

        let fontSize = button.titleLabel?.font.pointSize ?? 0
        button.titleLabel?.font = UIFont.systemFont(ofSize: fontSize, weight: .regular)

        container.addConstrainedSubviews([button]) {
            button.pinEdgesToSuperview(.init([.top(40), .trailing(16), .bottom(0), .leading(16)]))
        }

        container.directionalLayoutMargins = UIMetrics.SettingsCell.apiAccessInsetLayoutMargins

        return container
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let item = fetchedItems[indexPath.row]
        sendEdit(item: item)
    }

    private func configureNavigationItem() {
        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "Settings",
            value: "API access",
            comment: ""
        )
    }

    private func configureDataSource() {
        dataSource = UITableViewDiffableDataSource(
            tableView: tableView,
            cellProvider: { [weak self] _, indexPath, itemIdentifier in
                self?.dequeueCell(at: indexPath, itemIdentifier: itemIdentifier)
            }
        )
        updateDataSource(animated: false)
    }

    private func updateDataSource(animated: Bool = true) {
        let oldFetchedItems = fetchedItems
        let newFetchedItems = interactor.fetch()
        fetchedItems = newFetchedItems

        var snapshot = NSDiffableDataSourceSnapshot<ListAccessMethodSectionIdentifier, ListAccessMethodItemIdentifier>()
        snapshot.appendSections([.primary])

        let itemIdentifiers = newFetchedItems.map { item in
            ListAccessMethodItemIdentifier(id: item.id)
        }
        snapshot.appendItems(itemIdentifiers, toSection: .primary)

        for newFetchedItem in newFetchedItems {
            for oldFetchedItem in oldFetchedItems {
                if newFetchedItem.id == oldFetchedItem.id,
                   newFetchedItem.name != oldFetchedItem.name || newFetchedItem.detail != oldFetchedItem.detail {
                    snapshot.reloadItems([ListAccessMethodItemIdentifier(id: newFetchedItem.id)])
                }
            }
        }

        dataSource?.apply(snapshot, animatingDifferences: animated)
    }

    private func dequeueCell(
        at indexPath: IndexPath,
        itemIdentifier: ListAccessMethodItemIdentifier
    ) -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: CellReuseIdentifier.default, for: indexPath)
        let item = fetchedItems[indexPath.row]

        var contentConfiguration = UIListContentConfiguration.mullvadValueCell(tableStyle: .plain)
        contentConfiguration.text = item.name
        contentConfiguration.secondaryText = item.detail
        cell.contentConfiguration = contentConfiguration

        if let cell = cell as? DynamicBackgroundConfiguration {
            cell.setAutoAdaptingBackgroundConfiguration(.mullvadListPlainCell(), selectionType: .dimmed)
        }

        if let cell = cell as? CustomCellDisclosureHandling {
            cell.disclosureType = .chevron
        }

        return cell
    }

    private func sendAddNew() {
        delegate?.controllerShouldAddNew(self)
    }

    private func sendAbout() {
        delegate?.controllerShouldShowAbout(self)
    }

    private func sendEdit(item: ListAccessMethodItem) {
        delegate?.controller(self, shouldEditItem: item)
    }
}

private enum CellReuseIdentifier: String, CaseIterable, CellIdentifierProtocol {
    case `default`

    var cellClass: AnyClass {
        switch self {
        case .default: BasicCell.self
        }
    }
}
