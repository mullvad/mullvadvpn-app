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
    typealias MethodSettingsDataSource = UITableViewDiffableDataSource<
        MethodSettingsSectionIdentifier,
        MethodSettingsItemIdentifier
    >

    private let subject: CurrentValueSubject<AccessMethodViewModel, Never>
    private let interactor: EditAccessMethodInteractorProtocol
    private var cancellables = Set<AnyCancellable>()
    private var alertPresenter: AlertPresenter
    private var inputValidationErrors: [AccessMethodFieldValidationError] = []
    private var contentValidationErrors: [AccessMethodFieldValidationError] = []
    private var dataSource: MethodSettingsDataSource?

    private lazy var cellConfiguration: MethodSettingsCellConfiguration = {
        var configuration = MethodSettingsCellConfiguration(tableView: tableView, subject: subject)
        configuration.onCancelTest = { [weak self] in
            self?.onCancelTest()
        }
        return configuration
    }()

    private lazy var dataSourceConfiguration: MethodSettingsDataSourceConfiguration? = {
        return dataSource.flatMap { dataSource in
            MethodSettingsDataSourceConfiguration(dataSource: dataSource)
        }
    }()

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

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)

        inputValidationErrors.removeAll()
        contentValidationErrors.removeAll()
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
        case .validationError:
            return contentValidationErrors.isEmpty
                ? UIMetrics.SettingsCell.apiAccessCellHeight
                : UITableView.automaticDimension
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
        case .validationError, .cancelTest:
            return 0
        }
    }

    override func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        return nil
    }

    override func tableView(_ tableView: UITableView, heightForFooterInSection section: Int) -> CGFloat {
        guard let sectionIdentifier = dataSource?.snapshot().sectionIdentifiers[section] else { return 0 }

        switch sectionIdentifier {
        case .name, .protocol, .cancelTest:
            return UITableView.automaticDimension
        case .proxyConfiguration, .validationError, .testingStatus:
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

    // MARK: - Data source handling

    private func configureDataSource() {
        tableView.registerReusableViews(from: AccessMethodCellReuseIdentifier.self)
        tableView.registerReusableViews(from: AccessMethodHeaderFooterReuseIdentifier.self)

        dataSource = UITableViewDiffableDataSource(
            tableView: tableView,
            cellProvider: { [weak self] _, indexPath, itemIdentifier in
                guard let self else { return nil }

                return cellConfiguration.dequeueCell(
                    at: indexPath,
                    for: itemIdentifier,
                    contentValidationErrors: contentValidationErrors
                )
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
        let previousValidationError = inputValidationErrors

        validateInput()
        dataSourceConfiguration?.updateDataSource(
            previousValue: previousValue,
            newValue: newValue,
            previousValidationError: previousValidationError,
            newValidationError: inputValidationErrors,
            animated: animated
        )
    }

    private func configureTableView() {
        tableView.delegate = self
        tableView.backgroundColor = .secondaryColor
        tableView.separatorColor = .secondaryColor
        tableView.separatorInset.left = UIMetrics.SettingsCell.apiAccessInsetLayoutMargins.leading
    }

    // MARK: - Misc

    private func validateInput() {
        let validationResult = Result { try subject.value.validate() }
        let validationError = validationResult.error as? AccessMethodValidationError

        // Only look for empty values for input validation.
        inputValidationErrors = validationError?.fieldErrors.filter { error in
            error.kind == .emptyValue
        } ?? []

        saveBarButton.isEnabled = !isTesting && inputValidationErrors.isEmpty
    }

    private func validateContent() {
        let validationResult = Result { try subject.value.validate() }
        let validationError = validationResult.error as? AccessMethodValidationError

        // Only look for format errors for test(save validation.
        contentValidationErrors = validationError?.fieldErrors.filter { error in
            error.kind != .emptyValue
        } ?? []
    }

    private func onSave(transitionDelay: Duration = .zero) {
        interactor.saveAccessMethod()

        DispatchQueue.main.asyncAfter(deadline: .now() + transitionDelay.timeInterval) { [weak self] in
            guard let self else { return }
            delegate?.viewModelDidSave(subject.value)
        }
    }

    private func onTest() {
        validateContent()

        guard contentValidationErrors.isEmpty else {
            dataSourceConfiguration?.updateDataSourceWithContentValidationErrors(viewModel: subject.value)
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
}
