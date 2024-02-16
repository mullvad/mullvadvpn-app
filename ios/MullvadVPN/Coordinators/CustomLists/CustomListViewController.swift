//
//  CustomListViewController.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-15.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import MullvadSettings
import UIKit

protocol CustomListViewControllerDelegate: AnyObject {
    func customListDidSave()
    func customListDidDelete()
    func showLocations()
}

class CustomListViewController: UIViewController {
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
            title: NSLocalizedString(
                "CUSTOM_LIST_NAVIGATION_SAVE_BUTTON",
                tableName: "CustomLists",
                value: "Save",
                comment: ""
            ),
            primaryAction: UIAction { _ in
                self.onSave()
            }
        )
        barButtonItem.style = .done

        return barButtonItem
    }()

    weak var delegate: CustomListViewControllerDelegate?

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
        }.store(in: &cancellables)
    }

    private func configureNavigationItem() {
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
        cellConfiguration.onDelete = {
            self.onDelete()
        }

        dataSource = DataSource(
            tableView: tableView,
            cellProvider: { _, indexPath, itemIdentifier in
                self.cellConfiguration.dequeueCell(at: indexPath, for: itemIdentifier)
            }
        )

        dataSourceConfiguration?.didSelectItem = { item in
            self.view.endEditing(false)

            switch item {
            case .name, .deleteList:
                break
            case .addLocations, .editLocations:
                self.delegate?.showLocations()
            }
        }

        dataSourceConfiguration?.updateDataSource(sections: subject.value.tableSections, animated: false)
    }

    private func addSubviews() {
        view.addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview()
        }
    }

    private func onSave() {
        do {
            try interactor.createCustomList(viewModel: subject.value)
            delegate?.customListDidSave()
        } catch {
            // TODO: Show error dialog.
        }
    }

    private func onDelete() {
        // TODO: Show error dialog.
        delegate?.customListDidDelete()
    }
}
