//
//  MethodSettingsViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 21/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Combine
import struct MullvadTypes.Duration
import UIKit

/// The view controller providing the interface for editing method settings
/// and testing the proxy configuration.
class MethodSettingsViewController: UITableViewController {
    private let subject: CurrentValueSubject<AccessMethodViewModel, Never>
    private let interactor: EditAccessMethodInteractorProtocol
    private var validationError: AccessMethodValidationError?
    private var cancellables = Set<AnyCancellable>()
    private var alertPresenter: AlertPresenter

    private var dataSource: UITableViewDiffableDataSource<
        MethodSettingsSectionIdentifier,
        MethodSettingsItemIdentifier
    >?

    private var isTesting: Bool {
        subject.value.testingStatus == .inProgress
    }

    lazy var saveBarButton: UIBarButtonItem = {
        let barButtonItem = UIBarButtonItem(
            title: NSLocalizedString("SAVE_NAVIGATION_BUTTON", tableName: "APIAccess", value: "Save", comment: ""),
            primaryAction: UIAction { [weak self] _ in
                self?.onTest()
            }
        )
        barButtonItem.style = .done
        return barButtonItem
    }()

    weak var delegate: MethodSettingsViewControllerDelegate?

    init(
        subject: CurrentValueSubject<AccessMethodViewModel, Never>,
        interactor: EditAccessMethodInteractorProtocol,
        alertPresenter: AlertPresenter
    ) {
        self.subject = subject
        self.interactor = interactor
        self.alertPresenter = alertPresenter

        super.init(style: .insetGrouped)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.directionalLayoutMargins = UIMetrics.contentLayoutMargins
        view.backgroundColor = .secondaryColor

        navigationItem.rightBarButtonItem = saveBarButton
        isModalInPresentation = true

        configureTableView()
        configureDataSource()
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        interactor.cancelProxyConfigurationTest()
    }

    // MARK: - UITableViewDelegate

    override func tableView(_ tableView: UITableView, heightForRowAt indexPath: IndexPath) -> CGFloat {
        guard let itemIdentifier = dataSource?.itemIdentifier(for: indexPath) else { return 0 }

        switch itemIdentifier {
        case .name, .protocol, .proxyConfiguration, .cancelTest:
            return UIMetrics.SettingsCell.apiAccessCellHeight
        case .testingStatus:
            return UITableView.automaticDimension
        }
    }

    override func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        guard let sectionIdentifier = dataSource?.snapshot().sectionIdentifiers[section] else { return nil }

        guard let headerView = tableView
            .dequeueReusableView(withIdentifier: AccessMethodHeaderFooterReuseIdentifier.primary)
        else { return nil }

        var contentConfiguration = UIListContentConfiguration.mullvadGroupedHeader(tableStyle: tableView.style)
        contentConfiguration.text = sectionIdentifier.sectionName

        headerView.contentConfiguration = contentConfiguration

        return headerView
    }

    override func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        guard let sectionIdentifier = dataSource?.snapshot().sectionIdentifiers[section] else { return 0 }

        switch sectionIdentifier {
        case .name, .protocol, .proxyConfiguration, .testingStatus:
            return UITableView.automaticDimension
        case .cancelTest:
            return 0
        }
    }

    override func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        return nil
    }

    override func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        guard let sectionIdentifier = dataSource?.snapshot().sectionIdentifiers[section] else { return 0 }

        switch sectionIdentifier {
        case .name, .protocol, .proxyConfiguration, .cancelTest:
            return UITableView.automaticDimension
        case .testingStatus:
            return 0
        }
    }

    override func tableView(_ tableView: UITableView, willSelectRowAt indexPath: IndexPath) -> IndexPath? {
        guard !isTesting, let itemIdentifier = dataSource?.itemIdentifier(for: indexPath),
              itemIdentifier.isSelectable else { return nil }

        return indexPath
    }

    override func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        guard let itemIdentifier = dataSource?.itemIdentifier(for: indexPath) else { return false }

        return itemIdentifier.isSelectable
    }

    override func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        let itemIdentifier = dataSource?.itemIdentifier(for: indexPath)

        switch itemIdentifier {
        case .protocol:
            showProtocolSelector()
        case .proxyConfiguration(.shadowsocks(.cipher)):
            showShadowsocksCipher()
        default:
            break
        }
    }

    // MARK: - Pickers handling

    private func showProtocolSelector() {
        view.endEditing(false)
        delegate?.controllerShouldShowProtocolPicker(self)
    }

    private func showShadowsocksCipher() {
        view.endEditing(false)
        delegate?.controllerShouldShowShadowsocksCipherPicker(self)
    }

    // MARK: - Cell configuration

    private func dequeueCell(at indexPath: IndexPath, for itemIdentifier: MethodSettingsItemIdentifier)
        -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: itemIdentifier.cellIdentifier, for: indexPath)

        configureBackground(cell: cell, itemIdentifier: itemIdentifier)

        switch itemIdentifier {
        case .name:
            configureName(cell, itemIdentifier: itemIdentifier)
        case .protocol:
            configureProtocol(cell, itemIdentifier: itemIdentifier)
        case let .proxyConfiguration(proxyItemIdentifier):
            configureProxy(cell, itemIdentifier: proxyItemIdentifier)
        case .testingStatus:
            configureTestingStatus(cell, itemIdentifier: itemIdentifier)
        case .cancelTest:
            configureCancelTest(cell, itemIdentifier: itemIdentifier)
        }

        return cell
    }

    private func configureBackground(cell: UITableViewCell, itemIdentifier: MethodSettingsItemIdentifier) {
        configureErrorState(cell: cell, itemIdentifier: itemIdentifier)

        guard let cell = cell as? DynamicBackgroundConfiguration else { return }

        guard !itemIdentifier.isClearBackground else {
            cell.setAutoAdaptingClearBackgroundConfiguration()
            return
        }

        cell.setAutoAdaptingBackgroundConfiguration(.mullvadListGroupedCell(), selectionType: .dimmed)
    }

    private func configureErrorState(cell: UITableViewCell, itemIdentifier: MethodSettingsItemIdentifier) {
        guard case .proxyConfiguration = itemIdentifier else {
            return
        }

        // Only look for errors that are not empty values.
        let fieldErrors = validationError?.fieldErrors.filter { error in
            error.kind != .emptyValue
        }

        let errorsExist = fieldErrors?.isEmpty == false

        if errorsExist {
            cell.layer.cornerRadius = 10
            cell.layer.borderWidth = 1
            cell.layer.borderColor = UIColor.Cell.validationErrorBorderColor.cgColor
        } else {
            cell.layer.borderWidth = 0
        }
    }

    private func configureName(_ cell: UITableViewCell, itemIdentifier: MethodSettingsItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.setPlaceholder(type: .optional)
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        contentConfiguration.inputText = subject.value.name
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.name)

        cell.setDisabled(isTesting)
        cell.contentConfiguration = contentConfiguration
    }

    private func configureProxy(_ cell: UITableViewCell, itemIdentifier: ProxyProtocolConfigurationItemIdentifier) {
        switch itemIdentifier {
        case let .socks(socksItemIdentifier):
            let section = SocksSectionHandler(tableStyle: tableView.style, subject: subject)
            section.configure(cell, itemIdentifier: socksItemIdentifier)

        case let .shadowsocks(shadowsocksItemIdentifier):
            let section = ShadowsocksSectionHandler(tableStyle: tableView.style, subject: subject)
            section.configure(cell, itemIdentifier: shadowsocksItemIdentifier)
        }

        cell.setDisabled(isTesting)
    }

    private func configureProtocol(_ cell: UITableViewCell, itemIdentifier: MethodSettingsItemIdentifier) {
        var contentConfiguration = UIListContentConfiguration.mullvadValueCell(
            tableStyle: tableView.style,
            isEnabled: !isTesting
        )
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.secondaryText = subject.value.method.localizedDescription
        cell.contentConfiguration = contentConfiguration

        if let cell = cell as? CustomCellDisclosureHandling {
            cell.disclosureType = .chevron
        }

        cell.setDisabled(isTesting)
    }

    private func configureCancelTest(_ cell: UITableViewCell, itemIdentifier: MethodSettingsItemIdentifier) {
        var contentConfiguration = ButtonCellContentConfiguration()
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.isEnabled = isTesting
        contentConfiguration.primaryAction = UIAction { [weak self] _ in
            self?.onCancelTest()
        }

        cell.contentConfiguration = contentConfiguration
    }

    private func configureTestingStatus(_ cell: UITableViewCell, itemIdentifier: MethodSettingsItemIdentifier) {
        let viewStatus = subject.value.testingStatus.viewStatus

        var contentConfiguration = MethodTestingStatusCellContentConfiguration()
        contentConfiguration.status = viewStatus
        contentConfiguration.detailText = viewStatus == .reachable
            ? NSLocalizedString(
                "METHOD_SETTINGS_SAVING_CHANGES",
                tableName: "APIAccess",
                value: "Saving changes...",
                comment: ""
            )
            : nil

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

        validateOnInput()
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
        var snapshot = NSDiffableDataSourceSnapshot<MethodSettingsSectionIdentifier, MethodSettingsItemIdentifier>()

        // Add name field for user-defined access methods.
        if !newValue.method.isPermanent {
            snapshot.appendSections([.name])
            snapshot.appendItems([.name], toSection: .name)
        }

        snapshot.appendSections([.protocol])
        snapshot.appendItems([.protocol], toSection: .protocol)
        // Reconfigure protocol cell on change.
        if let previousValue, previousValue.method != newValue.method {
            snapshot.reconfigureOrReloadItems([.protocol])
        }

        // Add proxy configuration section if the access method is configurable.
        if newValue.method.hasProxyConfiguration {
            snapshot.appendSections([.proxyConfiguration])
        }

        switch newValue.method {
        case .direct, .bridges:
            break

        case .shadowsocks:
            snapshot.appendItems(MethodSettingsItemIdentifier.allShadowsocksItems, toSection: .proxyConfiguration)
            // Reconfigure cipher cell on change.
            if let previousValue, previousValue.shadowsocks.cipher != newValue.shadowsocks.cipher {
                snapshot.reconfigureOrReloadItems([.proxyConfiguration(.shadowsocks(.cipher))])
            }

            // Reconfigure the proxy configuration cell if validation error changed.
            if previousValidationError != newValidationError {
                snapshot.reconfigureOrReloadItems(MethodSettingsItemIdentifier.allShadowsocksItems)
            }
        case .socks5:
            snapshot.appendItems(
                MethodSettingsItemIdentifier.allSocksItems(authenticate: newValue.socks.authenticate),
                toSection: .proxyConfiguration
            )

            // Reconfigure the proxy configuration cell if validation error changed.
            if previousValidationError != newValidationError {
                snapshot.reconfigureOrReloadItems(
                    MethodSettingsItemIdentifier.allSocksItems(authenticate: newValue.socks.authenticate)
                )
            }
        }

        snapshot.appendSections([.testingStatus])
        snapshot.appendSections([.cancelTest])

        // Add test status below the test button.
        if newValue.testingStatus != .initial {
            snapshot.appendItems([.testingStatus], toSection: .testingStatus)

            // Show cancel test button below test status.
            if newValue.testingStatus == .inProgress {
                snapshot.appendItems([.cancelTest], toSection: .cancelTest)
            }
        }

        if let previousValue, previousValue.testingStatus != newValue.testingStatus {
            snapshot.reconfigureOrReloadItems(snapshot.itemIdentifiers)
        }

        dataSource?.apply(snapshot, animatingDifferences: animated)
    }

    private func validateOnInput() {
        let validationResult = Result { try subject.value.validate() }
        let validationError = validationResult.error as? AccessMethodValidationError

        // Only look for empty values for input validation.
        let fieldErrors = validationError?.fieldErrors.filter { error in
            error.kind == .emptyValue
        }

        let emptyValuesExist = fieldErrors?.isEmpty == false

        if emptyValuesExist {
            self.validationError = AccessMethodValidationError(fieldErrors: fieldErrors ?? [])
        } else {
            self.validationError = nil
        }

        saveBarButton.isEnabled = !emptyValuesExist
    }

    private func validateOnTest() {
        let validationResult = Result { try subject.value.validate() }
        validationError = validationResult.error as? AccessMethodValidationError
    }

    // MARK: - Misc

    private func configureTableView() {
        tableView.delegate = self
        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset.left = UIMetrics.SettingsCell.apiAccessInsetLayoutMargins.leading
    }

    private func onSave(transitionDelay: Duration = .zero) {
        interactor.saveAccessMethod()

        DispatchQueue.main.asyncAfter(deadline: .now() + transitionDelay.timeInterval) { [weak self] in
            guard let self else { return }
            delegate?.controllerDidSaveAccessMethod(self)
        }
    }

    private func onTest() {
        validateOnTest()

        if
            let validationError,
            var snapshot = dataSource?.snapshot() {
            let itemsToReload = MethodSettingsItemIdentifier.fromValidationErrors(
                validationError,
                selectedMethod: subject.value.method
            )

            snapshot.reconfigureOrReloadItems(itemsToReload)
            dataSource?.apply(snapshot, animatingDifferences: true)

            return
        }

        view.endEditing(true)
        saveBarButton.isEnabled = false

        interactor.startProxyConfigurationTest { [weak self] _ in
            self?.onTestCompleted()
        }
    }

    private func onTestCompleted() {
        switch subject.value.testingStatus {
        case .initial, .inProgress:
            break

        case .failed:
            let presentation = AlertPresentation(
                id: "api-access-methods-testing-status-failed-alert",
                icon: .warning,
                message: NSLocalizedString(
                    "METHOD_SETTINGS_SAVE_PROMPT",
                    tableName: "APIAccess",
                    value: "API could not be reached, save anyway?",
                    comment: ""
                ),
                buttons: [
                    AlertAction(
                        title: NSLocalizedString(
                            "METHOD_SETTINGS_SAVE_BUTTON",
                            tableName: "APIAccess",
                            value: "Save anyway",
                            comment: ""
                        ),
                        style: .default,
                        handler: { [weak self] in
                            self?.onSave()
                        }
                    ),
                    AlertAction(
                        title: NSLocalizedString(
                            "METHOD_SETTINGS_BACK_BUTTON",
                            tableName: "APIAccess",
                            value: "Back to editing",
                            comment: ""
                        ),
                        style: .default
                    ),
                ]
            )

            alertPresenter.showAlert(presentation: presentation, animated: true)
        case .succeeded:
            onSave(transitionDelay: .seconds(1))
        }
    }

    private func onCancelTest() {
        interactor.cancelProxyConfigurationTest()
    }

    // swiftlint:disable:next file_length
}
