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

    private var persistedCustomList: CustomList? {
        return interactor.fetchAll().first(where: { $0.id == subject.value.id })
    }

    private var hasUnsavedChanges: Bool {
        persistedCustomList != subject.value.customList
    }

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
            primaryAction: UIAction { [weak self] _ in
                self?.onSave()
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
        if let navigationController = navigationController as? InterceptibleNavigationController {
            interceptNavigation(navigationController)
        }

        navigationController?.interactivePopGestureRecognizer?.delegate = self
        navigationItem.rightBarButtonItem = saveBarButton
    }

    private func interceptNavigation(_ navigationController: InterceptibleNavigationController) {
        navigationController.shouldPopViewController = { [weak self] viewController in
            guard
                let self,
                viewController is Self,
                hasUnsavedChanges
            else { return true }

            self.onUnsavedChanges()
            return false
        }

        navigationController.shouldPopToViewController = { [weak self] viewController in
            guard
                let self,
                viewController is ListCustomListViewController,
                hasUnsavedChanges
            else { return true }

            self.onUnsavedChanges()
            return false
        }
    }

    private func configureTableView() {
        tableView.delegate = dataSourceConfiguration
        tableView.backgroundColor = .secondaryColor
        tableView.registerReusableViews(from: CustomListItemIdentifier.CellIdentifier.self)
    }

    private func configureDataSource() {
        cellConfiguration.onDelete = { [weak self] in
            self?.onDelete()
        }

        dataSource = DataSource(
            tableView: tableView,
            cellProvider: { [weak self] _, indexPath, itemIdentifier in
                guard let self else { return nil }
                return cellConfiguration.dequeueCell(
                    at: indexPath,
                    for: itemIdentifier,
                    validationErrors: self.validationErrors
                )
            }
        )

        dataSourceConfiguration?.didSelectItem = { [weak self] item in
            guard let self else { return }
            self.view.endEditing(false)

            switch item {
            case .name, .deleteList:
                break
            case .addLocations, .editLocations:
                delegate?.showLocations(self.subject.value.customList)
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

    @objc private func onUnsavedChanges() {
        let message = NSMutableAttributedString(
            markdownString: NSLocalizedString(
                "CUSTOM_LISTS_UNSAVED_CHANGES_PROMPT",
                tableName: "CustomLists",
                value: "You have unsaved changes.",
                comment: ""
            ),
            options: MarkdownStylingOptions(font: .preferredFont(forTextStyle: .body))
        )

        let presentation = AlertPresentation(
            id: "api-custom-lists-unsaved-changes-alert",
            icon: .alert,
            attributedMessage: message,
            buttons: [
                AlertAction(
                    title: NSLocalizedString(
                        "CUSTOM_LISTS_DISCARD_CHANGES_BUTTON",
                        tableName: "CustomLists",
                        value: "Discard changes",
                        comment: ""
                    ),
                    style: .destructive,
                    handler: {
                        // Reset subject/view model to no longer having unsaved changes.
                        if let persistedCustomList = self.persistedCustomList {
                            self.subject.value.update(with: persistedCustomList)
                        }
                        self.delegate?.customListDidSave(self.subject.value.customList)
                    }
                ),
                AlertAction(
                    title: NSLocalizedString(
                        "CUSTOM_LISTS_BACK_TO_EDITING_BUTTON",
                        tableName: "CustomLists",
                        value: "Back to editing",
                        comment: ""
                    ),
                    style: .default
                ),
            ]
        )

        alertPresenter.showAlert(presentation: presentation, animated: true)
    }
}

extension CustomListViewController: UIGestureRecognizerDelegate {
    // For some reason, intercepting `popViewController(animated: Bool)` in `InterceptibleNavigationController`
    // by SWIPING back leads to weird behaviour where subsequent navigation seem to happen systemwise but not
    // UI-wise. This leads to the UI freezing up, and the only remedy is to restart the app.
    //
    // To get around this issue we can intercept the back swipe gesture and manually perform the transition
    // instead, thereby bypassing the inner mechanisms that seem to go out of sync.
    func gestureRecognizerShouldBegin(_ gestureRecognizer: UIGestureRecognizer) -> Bool {
        guard gestureRecognizer == navigationController?.interactivePopGestureRecognizer else {
            return true
        }

        navigationController?.popViewController(animated: true)
        return false
    }
}
