//
//  CustomListCellConfiguration.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

struct CustomListCellConfiguration {
    let tableView: UITableView
    let subject: CurrentValueSubject<CustomListViewModel, Never>

    var onDelete: (() -> Void)?

    func dequeueCell(
        at indexPath: IndexPath,
        for itemIdentifier: CustomListItemIdentifier,
        validationErrors: Set<CustomListFieldValidationError>
    ) -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: itemIdentifier.cellIdentifier, for: indexPath)

        configureBackground(cell: cell, itemIdentifier: itemIdentifier, validationErrors: validationErrors)

        switch itemIdentifier {
        case .name:
            configureName(cell, itemIdentifier: itemIdentifier)
        case .addLocations, .editLocations:
            configureLocations(cell, itemIdentifier: itemIdentifier)
        case .deleteList:
            configureDelete(cell, itemIdentifier: itemIdentifier)
        }

        return cell
    }

    private func configureBackground(
        cell: UITableViewCell,
        itemIdentifier: CustomListItemIdentifier,
        validationErrors: Set<CustomListFieldValidationError>
    ) {
        configureErrorState(
            cell: cell,
            itemIdentifier: itemIdentifier,
            contentValidationErrors: validationErrors
        )

        guard let cell = cell as? DynamicBackgroundConfiguration else { return }

        cell.setAutoAdaptingBackgroundConfiguration(.mullvadListGroupedCell(), selectionType: .dimmed)
    }

    private func configureErrorState(
        cell: UITableViewCell,
        itemIdentifier: CustomListItemIdentifier,
        contentValidationErrors: Set<CustomListFieldValidationError>
    ) {
        let itemsWithErrors = CustomListItemIdentifier.fromFieldValidationErrors(contentValidationErrors)

        if itemsWithErrors.contains(itemIdentifier) {
            cell.layer.cornerRadius = 10
            cell.layer.borderWidth = 1
            cell.layer.borderColor = UIColor.Cell.validationErrorBorderColor.cgColor
        } else {
            cell.layer.borderWidth = 0
        }
    }

    private func configureName(_ cell: UITableViewCell, itemIdentifier: CustomListItemIdentifier) {
        var contentConfiguration = TextCellContentConfiguration()

        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.setPlaceholder(type: .required)
        contentConfiguration.textFieldProperties = .withSmartFeaturesDisabled()
        contentConfiguration.inputText = subject.value.name
        contentConfiguration.editingEvents.onChange = subject.bindTextAction(to: \.name)

        cell.contentConfiguration = contentConfiguration
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

    private func configureLocations(_ cell: UITableViewCell, itemIdentifier: CustomListItemIdentifier) {
        var contentConfiguration = UIListContentConfiguration.mullvadValueCell(tableStyle: tableView.style)

        contentConfiguration.text = itemIdentifier.text
        cell.contentConfiguration = contentConfiguration

        if let cell = cell as? CustomCellDisclosureHandling {
            cell.disclosureType = .chevron
        }
    }

    private func configureDelete(_ cell: UITableViewCell, itemIdentifier: CustomListItemIdentifier) {
        var contentConfiguration = ButtonCellContentConfiguration()

        contentConfiguration.style = .tableInsetGroupedDanger
        contentConfiguration.text = itemIdentifier.text
        contentConfiguration.primaryAction = UIAction { _ in
            onDelete?()
        }

        cell.contentConfiguration = contentConfiguration
    }
}
