//
//  ListAccessMethodViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 02/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadREST
import MullvadSettings
import UIKit

enum ListAccessMethodSectionIdentifier: Hashable {
    case primary
}

struct ListAccessMethodItemIdentifier: Hashable {
    var id: UUID
}

/// View controller presenting a list of API access methods.
class ListAccessMethodViewController: UIViewController, UITableViewDelegate {
    typealias ListAccessMethodDataSource = UITableViewDiffableDataSource<
        ListAccessMethodSectionIdentifier,
        ListAccessMethodItemIdentifier
    >

    private let headerView = ListAccessMethodHeaderView()
    private let interactor: ListAccessMethodInteractorProtocol
    private var lastReachableMethodItem: ListAccessMethodItem?
    private var cancellables = Set<AnyCancellable>()

    private var dataSource: ListAccessMethodDataSource?
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

        interactor.itemsPublisher.sink { [weak self] _ in
            self?.updateDataSource(animated: true)
        }
        .store(in: &cancellables)

        interactor.itemInUsePublisher.sink { [weak self] item in
            self?.lastReachableMethodItem = item
            self?.updateDataSource(animated: true)
        }
        .store(in: &cancellables)

        configureNavigationItem()
        configureDataSource()
    }

    func tableView(_ tableView: UITableView, heightForRowAt indexPath: IndexPath) -> CGFloat {
        guard let itemIdentifier = dataSource?.itemIdentifier(for: indexPath) else { return 0 }

        if itemIdentifier.id == lastReachableMethodItem?.id {
            return UITableView.automaticDimension
        } else {
            return UIMetrics.SettingsCell.apiAccessCellHeight
        }
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
        dataSource = ListAccessMethodDataSource(
            tableView: tableView,
            cellProvider: { [weak self] _, indexPath, itemIdentifier in
                self?.dequeueCell(at: indexPath, itemIdentifier: itemIdentifier)
            }
        )
        updateDataSource(animated: false)
    }

    private func updateDataSource(animated: Bool = true) {
        fetchedItems = interactor.fetch()

        var snapshot = NSDiffableDataSourceSnapshot<ListAccessMethodSectionIdentifier, ListAccessMethodItemIdentifier>()
        snapshot.appendSections([.primary])

        let itemIdentifiers = fetchedItems.map { item in
            ListAccessMethodItemIdentifier(id: item.id)
        }
        snapshot.appendItems(itemIdentifiers, toSection: .primary)

        for item in fetchedItems {
            snapshot.reloadItems([ListAccessMethodItemIdentifier(id: item.id)])
        }

        dataSource?.apply(snapshot, animatingDifferences: animated)
    }

    private func dequeueCell(
        at indexPath: IndexPath,
        itemIdentifier: ListAccessMethodItemIdentifier
    ) -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: CellReuseIdentifier.default, for: indexPath)
        let item = fetchedItems[indexPath.row]

        var contentConfiguration = ListCellContentConfiguration()
        contentConfiguration.text = item.name
        contentConfiguration.secondaryText = item.detail
        contentConfiguration.tertiaryText = lastReachableMethodItem?.id == item.id
            ? NSLocalizedString("LIST_ACCESS_METHODS_IN_USE_ITEM", tableName: "APIAccess", value: "In use", comment: "")
            : ""
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
