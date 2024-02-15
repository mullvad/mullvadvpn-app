//
//  AddCustomListViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import UIKit

protocol AddCustomListViewControllerDelegate: AnyObject {
    func customListDidSave()
    func showLocations()
}

class AddCustomListViewController: UIViewController {
    typealias DataSource = UITableViewDiffableDataSource<CustomListSectionIdentifier, CustomListItemIdentifier>

    private let interactor: CustomListInteractorProtocol
    private let tableView = UITableView(frame: .zero, style: .insetGrouped)
    private let subject: CurrentValueSubject<CustomListViewModel, Never>
    private var cancellables = Set<AnyCancellable>()
    private var dataSource: DataSource?

    private lazy var cellConfiguration: CustomListCellConfiguration = {
        CustomListCellConfiguration(tableView: tableView, subject: subject)
    }()

    private lazy var dataSourceConfiguration: CustomListDataSourceConfiguration? = {
        dataSource.flatMap { dataSource in
            CustomListDataSourceConfiguration(dataSource: dataSource)
        }
    }()

    lazy var saveBarButton: UIBarButtonItem = {
        let barButtonItem = UIBarButtonItem(
            title: NSLocalizedString("SAVE_NAVIGATION_BUTTON", tableName: "CustomLists", value: "Create", comment: ""),
            primaryAction: UIAction { _ in
                self.onSave()
            }
        )
        barButtonItem.style = .done
        return barButtonItem
    }()

    weak var delegate: AddCustomListViewControllerDelegate?

    init(
        interactor: CustomListInteractorProtocol,
        subject: CurrentValueSubject<CustomListViewModel, Never>
    ) {
        self.subject = subject
        self.interactor = interactor

        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins
        view.backgroundColor = .secondaryColor
        isModalInPresentation = true

        addSubviews()
        configureNavigationItem()
        configureDataSource()
        configureTableView()

        subject.sink { viewModel in
            self.saveBarButton.isEnabled = !viewModel.name.isEmpty
        }
        .store(in: &cancellables)
    }

    private func configureNavigationItem() {
        navigationItem.title = NSLocalizedString(
            "CUSTOM_LIST_NAVIGATION_ADD_TITLE",
            tableName: "CustomLists",
            value: "New custom list",
            comment: ""
        )

        navigationItem.leftBarButtonItem = UIBarButtonItem(
            systemItem: .cancel,
            primaryAction: UIAction(handler: { _ in
                self.dismiss(animated: true)
            })
        )

        navigationItem.rightBarButtonItem = saveBarButton
    }

    private func configureTableView() {
        tableView.delegate = dataSourceConfiguration
        tableView.backgroundColor = .secondaryColor
        tableView.registerReusableViews(from: CustomListItemIdentifier.CellIdentifier.self)
    }

    private func configureDataSource() {
        dataSource = DataSource(
            tableView: tableView,
            cellProvider: { _, indexPath, itemIdentifier in
                self.cellConfiguration.dequeueCell(at: indexPath, for: itemIdentifier)
            }
        )

        dataSourceConfiguration?.didSelectItem = { item in
            switch item {
            case .name:
                break
            case .locations:
                self.showLocations()
            }
        }

        dataSourceConfiguration?.updateDataSource(animated: false)
    }

    private func addSubviews() {
        view.addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview()
        }
    }

    private func showLocations() {
        view.endEditing(false)
        delegate?.showLocations()
    }

    private func onSave() {
        do {
            try interactor.createCustomList(viewModel: subject.value)
            delegate?.customListDidSave()
        } catch {
            // TODO: Show error dialog.
        }
    }
}
