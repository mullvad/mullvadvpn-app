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
    func customListDidSave(_ list: CustomList)
    func customListDidDelete(_ list: CustomList)
    func showLocations(_ list: CustomList)
}

class CustomListViewController: UIViewController {
    typealias DataSource = UITableViewDiffableDataSource<CustomListSectionIdentifier, CustomListItemIdentifier>

    private let interactor: CustomListInteractorProtocol
    private let tableView = UITableView(frame: .zero, style: .insetGrouped)
    private let subject: CurrentValueSubject<CustomListViewModel, Never>
    private var cancellables = Set<AnyCancellable>()
    private var dataSource: DataSource?
    private let alertPresenter: AlertPresenter
    private var validationErrors: Set<CustomListFieldValidationError> = []

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
        subject: CurrentValueSubject<CustomListViewModel, Never>,
        alertPresenter: AlertPresenter
    ) {
        self.subject = subject
        self.interactor = interactor
        self.alertPresenter = alertPresenter

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

        subject.sink { [weak self] viewModel in
            self?.saveBarButton.isEnabled = !viewModel.name.isEmpty
            self?.validationErrors.removeAll()
        }.store(in: &cancellables)
    }

    private func configureNavigationItem() {
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
                self.cellConfiguration.dequeueCell(
                    at: indexPath,
                    for: itemIdentifier,
                    validationErrors: self.validationErrors
                )
            }
        )

        dataSourceConfiguration?.didSelectItem = { item in
            self.view.endEditing(false)

            switch item {
            case .name, .deleteList:
                break
            case .addLocations, .editLocations:
                self.delegate?.showLocations(self.subject.value.customList)
            }
        }

        dataSourceConfiguration?.updateDataSource(
            sections: subject.value.tableSections,
            validationErrors: validationErrors,
            animated: false
        )
    }

    private func addSubviews() {
        view.addConstrainedSubviews([tableView]) {
            tableView.pinEdgesToSuperview()
        }
    }

    private func onSave() {
        do {
            try interactor.save(viewModel: subject.value)
            delegate?.customListDidSave(subject.value.customList)
        } catch {
            validationErrors.insert(.name)
            dataSourceConfiguration?.set(validationErrors: validationErrors)
        }
    }

    private func onDelete() {
        let message = NSMutableAttributedString(
            markdownString: NSLocalizedString(
                "CUSTOM_LISTS_DELETE_PROMPT",
                tableName: "CustomLists",
                value: "Do you want to delete the list **\(subject.value.name)**?",
                comment: ""
            ),
            options: MarkdownStylingOptions(font: .preferredFont(forTextStyle: .body))
        )

        let presentation = AlertPresentation(
            id: "api-custom-lists-delete-list-alert",
            icon: .alert,
            attributedMessage: message,
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "CUSTOM_LISTS_DELETE_BUTTON",
                        tableName: "CustomLists",
                        value: "Delete list",
                        comment: ""
                    ),
                    style: .destructive,
                    handler: {
                        self.interactor.delete(id: self.subject.value.id)
                        self.delegate?.customListDidDelete(self.subject.value.customList)
                    }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "CUSTOM_LISTS_CANCEL_BUTTON",
                        tableName: "CustomLists",
                        value: "Cancel",
                        comment: ""
                    ),
                    style: .default
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }
}
