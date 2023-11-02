//
//  EditAccessMethodViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 17/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

/// The view controller providing the interface for editing the existing access method.
class EditAccessMethodViewController: UITableViewController {
    private let subject: CurrentValueSubject<AccessMethodViewModel, Never>
    private var validationError: AccessMethodValidationError?
    private let interactor: EditAccessMethodInteractorProtocol
    private var cancellables = Set<AnyCancellable>()
    private var dataSource: UITableViewDiffableDataSource<
        EditAccessMethodSectionIdentifier,
        EditAccessMethodItemIdentifier
    >?
    private lazy var saveBarButton: UIBarButtonItem = {
        let barButton = UIBarButtonItem(systemItem: .save, primaryAction: UIAction { [weak self] _ in
            self?.onSave()
        })
        barButton.style = .done
        return barButton
    }()

    weak var delegate: EditAccessMethodViewControllerDelegate?

    init(subject: CurrentValueSubject<AccessMethodViewModel, Never>, interactor: EditAccessMethodInteractorProtocol) {
        self.subject = subject
        self.interactor = interactor
        super.init(style: .insetGrouped)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor
        tableView.backgroundColor = .secondaryColor

        configureDataSource()
        configureNavigationItem()
    }

    override func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        guard let itemIdentifier = dataSource?.itemIdentifier(for: indexPath) else { return false }

        return itemIdentifier.isSelectable
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        guard let itemIdentifier = dataSource?.itemIdentifier(for: indexPath) else { return }

        if case .proxyConfiguration = itemIdentifier {
            delegate?.controllerShouldShowProxyConfiguration(self)
        }
    }

    override func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        guard let sectionIdentifier = dataSource?.snapshot().sectionIdentifiers[section] else { return nil }
        guard let sectionFooterText = sectionIdentifier.sectionFooter else { return nil }

        guard let headerView = tableView
            .dequeueReusableView(withIdentifier: AccessMethodHeaderFooterReuseIdentifier.primary)
        else { return nil }

        var contentConfiguration = UIListContentConfiguration.mullvadGroupedFooter()
        contentConfiguration.text = sectionFooterText

        headerView.contentConfiguration = contentConfiguration

        return headerView
    }

    // MARK: - Cell configuration

    private func dequeueCell(at indexPath: IndexPath, for itemIdentifier: EditAccessMethodItemIdentifier)
        -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: itemIdentifier.cellIdentifier, for: indexPath)

        configureBackground(cell: cell, itemIdentifier: itemIdentifier)

        switch itemIdentifier {
        case .name:
            configureName(cell, itemIdentifier: itemIdentifier)
        case .testMethod:
            configureTestMethod(cell, itemIdentifier: itemIdentifier)
        case .testingStatus:
            configureTestingStatus(cell, itemIdentifier: itemIdentifier)
        case .deleteMethod:
            configureDeleteMethod(cell, itemIdentifier: itemIdentifier)
        case .useIfAvailable:
            configureUseIfAvailable(cell, itemIdentifier: itemIdentifier)
        case .proxyConfiguration:
            configureProxyConfiguration(cell, itemIdentifier: itemIdentifier)
        }

        return cell
    }

    private func configureBackground(cell: UITableViewCell, itemIdentifier: EditAccessMethodItemIdentifier) {
        guard let cell = cell as? DynamicBackgroundConfiguration else { return }

        guard !itemIdentifier.isClearBackground else {
            cell.setAutoAdaptingClearBackgroundConfiguration()
            return
        }

        var backgroundConfiguration = UIBackgroundConfiguration.mullvadListGroupedCell()

        if case .proxyConfiguration = itemIdentifier, let validationError,
           validationError.containsProxyConfigurationErrors(selectedMethod: subject.value.method) {
            backgroundConfiguration.applyValidationErrorStyle()
        }

        cell.setAutoAdaptingBackgroundConfiguration(backgroundConfiguration, selectionType: .dimmed)
    }

    private func configureName(_ cell: UITableViewCell, itemIdentifier: EditAccessMethodItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.setPlaceholder(type: .optional)
        contentConfiguration.textFieldProperties = .withAutoResignAndDoneReturnKey()
        contentConfiguration.inputText = subject.value.name
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.name)
        cell.contentConfiguration = contentConfiguration
    }

    private func configureTestMethod(_ cell: UITableViewCell, itemIdentifier: EditAccessMethodItemIdentifier) {
        var contentConfiguration = ButtonCellContentConfiguration()
        contentConfiguration.style = .tableInsetGroupedSuccess
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.isEnabled = subject.value.testingStatus != .inProgress
        contentConfiguration.primaryAction = UIAction { [weak self] _ in
            self?.onTest()
        }
        cell.contentConfiguration = contentConfiguration
    }

    private func configureTestingStatus(_ cell: UITableViewCell, itemIdentifier: EditAccessMethodItemIdentifier) {
        var contentConfiguration = MethodTestingStatusCellContentConfiguration()
        contentConfiguration.sheetConfiguration = .init(status: subject.value.testingStatus.sheetStatus)
        cell.contentConfiguration = contentConfiguration
    }

    private func configureUseIfAvailable(_ cell: UITableViewCell, itemIdentifier: EditAccessMethodItemIdentifier) {
        var contentConfiguration = SwitchCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.isOn = subject.value.isEnabled
        contentConfiguration.onChange = subject.bindSwitchAction(to: \.isEnabled)
        cell.contentConfiguration = contentConfiguration
    }

    private func configureProxyConfiguration(_ cell: UITableViewCell, itemIdentifier: EditAccessMethodItemIdentifier) {
        var contentConfiguration = UIListContentConfiguration.mullvadCell(tableStyle: tableView.style)
        contentConfiguration.text = itemIdentifier.text
        cell.contentConfiguration = contentConfiguration

        if let cell = cell as? CustomCellDisclosureHandling {
            cell.disclosureType = .chevron
        }
    }

    private func configureDeleteMethod(_ cell: UITableViewCell, itemIdentifier: EditAccessMethodItemIdentifier) {
        var contentConfiguration = ButtonCellContentConfiguration()
        contentConfiguration.style = .tableInsetGroupedDanger
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.primaryAction = UIAction { [weak self] _ in
            self?.onDelete()
        }
        cell.contentConfiguration = contentConfiguration
    }

    // MARK: - Data source handling

    private func configureDataSource() {
        tableView.registerReusableViews(from: AccessMethodCellReuseIdentifier.self)
        tableView.registerReusableViews(from: AccessMethodHeaderFooterReuseIdentifier.self)

        dataSource = UITableViewDiffableDataSource(
            tableView: tableView,
            cellProvider: { [weak self] _, indexPath, itemIdentifier in
                self?.dequeueCell(at: indexPath, for: itemIdentifier)
            }
        )
        subject.withPreviousValue()
            .sink { [weak self] previousValue, newValue in
                self?.viewModelDidChange(previousValue: previousValue, newValue: newValue)
            }
            .store(in: &cancellables)
    }

    private func viewModelDidChange(previousValue: AccessMethodViewModel?, newValue: AccessMethodViewModel) {
        let animated = view.window != nil
        let previousValidationError = validationError

        validateViewModel()
        updateBarButtons()
        updateDataSource(
            previousValue: previousValue,
            newValue: newValue,
            previousValidationError: previousValidationError,
            newValidationError: validationError,
            animated: animated
        )
    }

    private func updateDataSource(
        previousValue: AccessMethodViewModel?,
        newValue: AccessMethodViewModel,
        previousValidationError: AccessMethodValidationError?,
        newValidationError: AccessMethodValidationError?,
        animated: Bool
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<EditAccessMethodSectionIdentifier, EditAccessMethodItemIdentifier>()

        // Add name field for user-defined access methods.
        if !newValue.method.isPermanent {
            snapshot.appendSections([.name])
            snapshot.appendItems([.name], toSection: .name)
        }

        // Add static sections.
        snapshot.appendSections([.testMethod, .useIfAvailable])

        snapshot.appendItems([.testMethod], toSection: .testMethod)
        // Reconfigure the test button on status changes.
        if let previousValue, previousValue.testingStatus != newValue.testingStatus {
            snapshot.reconfigureOrReloadItems([.testMethod])
        }

        // Add test status below the test button.
        if newValue.testingStatus != .initial {
            snapshot.appendItems([.testingStatus], toSection: .testMethod)
            if let previousValue, previousValue.testingStatus != newValue.testingStatus {
                snapshot.reconfigureOrReloadItems([.testingStatus])
            }
        }

        snapshot.appendItems([.useIfAvailable], toSection: .useIfAvailable)

        // Add proxy configuration if the access method is configurable.
        if newValue.method.hasProxyConfiguration {
            snapshot.appendSections([.proxyConfiguration])
            snapshot.appendItems([.proxyConfiguration], toSection: .proxyConfiguration)

            // Reconfigure the proxy configuration cell if validation error changed.
            if previousValidationError != newValidationError {
                snapshot.reconfigureOrReloadItems([.proxyConfiguration])
            }
        }

        // Add delete button for user-defined access methods.
        if !newValue.method.isPermanent {
            snapshot.appendSections([.deleteMethod])
            snapshot.appendItems([.deleteMethod], toSection: .deleteMethod)
        }

        dataSource?.apply(snapshot, animatingDifferences: animated)
    }

    // MARK: - Misc

    private func configureNavigationItem() {
        navigationItem.title = subject.value.navigationItemTitle
        navigationItem.rightBarButtonItem = saveBarButton
    }

    private func validateViewModel() {
        let validationResult = Result { try subject.value.validate() }
        validationError = validationResult.error as? AccessMethodValidationError
    }

    private func updateBarButtons() {
        saveBarButton.isEnabled = validationError == nil
    }

    private func onDelete() {
        interactor.deleteAccessMethod()
        delegate?.controllerDidDeleteAccessMethod(self)
    }

    private func onSave() {
        interactor.saveAccessMethod()
        delegate?.controllerDidSaveAccessMethod(self)
    }

    private func onTest() {
        interactor.startProxyConfigurationTest()
    }
}
