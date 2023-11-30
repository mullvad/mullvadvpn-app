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
        tableView.rowHeight = UITableView.automaticDimension
        tableView.estimatedRowHeight = 60

        tableView.registerReusableViews(from: CellReuseIdentifier.self)

        view.addConstrainedSubviews([headerView, tableView]) {
            headerView.pinEdgesToSuperview(.all().excluding(.bottom))
            tableView.pinEdgesToSuperview(.all().excluding(.top))
            tableView.topAnchor.constraint(equalTo: headerView.bottomAnchor)
        }

        addChild(contentController)
        contentController.didMove(toParent: self)

        interactor.publisher.sink { newElements in
            self.updateDataSource(animated: true)
        }
        .store(in: &cancellables)

        configureNavigationItem()
        configureDataSource()
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
        navigationItem.rightBarButtonItem = UIBarButtonItem(
            systemItem: .add,
            primaryAction: UIAction(handler: { [weak self] _ in
                self?.sendAddNew()
            })
        )
    }

    private func configureDataSource() {
        dataSource = UITableViewDiffableDataSource(
            tableView: tableView,
            cellProvider: { [weak self] tableView, indexPath, itemIdentifier in
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
