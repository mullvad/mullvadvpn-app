//
//  MethodSettingsCellConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-01-19.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

class MethodSettingsCellConfiguration {
    private let subject: CurrentValueSubject<AccessMethodViewModel, Never>
    private let tableView: UITableView

    var onCancelTest: (() -> Void)?

    private var isTesting: Bool {
        subject.value.testingStatus == .inProgress
    }

    init(tableView: UITableView, subject: CurrentValueSubject<AccessMethodViewModel, Never>) {
        self.tableView = tableView
        self.subject = subject
    }

    func dequeueCell(
        at indexPath: IndexPath,
        for itemIdentifier: MethodSettingsItemIdentifier,
        contentValidationErrors: [AccessMethodFieldValidationError]
    ) -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: itemIdentifier.cellIdentifier, for: indexPath)

        configureBackground(
            cell: cell,
            itemIdentifier: itemIdentifier,
            contentValidationErrors: contentValidationErrors
        )

        switch itemIdentifier {
        case .name:
            configureName(cell, itemIdentifier: itemIdentifier)
        case .protocol:
            configureProtocol(cell, itemIdentifier: itemIdentifier)
        case let .proxyConfiguration(proxyItemIdentifier):
            configureProxy(cell, itemIdentifier: proxyItemIdentifier)
        case .validationError:
            configureValidationError(
                cell,
                itemIdentifier: itemIdentifier,
                contentValidationErrors: contentValidationErrors
            )
        case .testingStatus:
            configureTestingStatus(cell, itemIdentifier: itemIdentifier)
        case .cancelTest:
            configureCancelTest(cell, itemIdentifier: itemIdentifier)
        }

        return cell
    }

    private func configureBackground(
        cell: UITableViewCell,
        itemIdentifier: MethodSettingsItemIdentifier,
        contentValidationErrors: [AccessMethodFieldValidationError]
    ) {
        configureErrorState(
            cell: cell,
            itemIdentifier: itemIdentifier,
            contentValidationErrors: contentValidationErrors
        )

        guard let cell = cell as? DynamicBackgroundConfiguration else { return }

        guard !itemIdentifier.isClearBackground else {
            cell.setAutoAdaptingClearBackgroundConfiguration()
            return
        }

        cell.setAutoAdaptingBackgroundConfiguration(.mullvadListGroupedCell(), selectionType: .dimmed)
    }

    private func configureErrorState(
        cell: UITableViewCell,
        itemIdentifier: MethodSettingsItemIdentifier,
        contentValidationErrors: [AccessMethodFieldValidationError]
    ) {
        guard case .proxyConfiguration = itemIdentifier else {
            return
        }

        let itemsWithErrors = MethodSettingsItemIdentifier.fromFieldValidationErrors(
            contentValidationErrors,
            selectedMethod: subject.value.method
        )

        if itemsWithErrors.contains(itemIdentifier) {
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
        contentConfiguration.setPlaceholder(type: .required)
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

    private func configureValidationError(
        _ cell: UITableViewCell,
        itemIdentifier: MethodSettingsItemIdentifier,
        contentValidationErrors: [AccessMethodFieldValidationError]
    ) {
        var contentConfiguration = MethodSettingsValidationErrorContentConfiguration()
        contentConfiguration.fieldErrors = contentValidationErrors

        cell.contentConfiguration = contentConfiguration
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
            self?.onCancelTest?()
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
}
