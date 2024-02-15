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

    func dequeueCell(at indexPath: IndexPath, for itemIdentifier: CustomListItemIdentifier) -> UITableViewCell {
        let cell = tableView.dequeueReusableView(withIdentifier: itemIdentifier.cellIdentifier, for: indexPath)

        configureBackground(cell: cell, itemIdentifier: itemIdentifier)

        switch itemIdentifier {
        case .name:
            configureName(cell, itemIdentifier: itemIdentifier)
        case .locations:
            configureLocations(cell, itemIdentifier: itemIdentifier)
        }

        return cell
    }

    private func configureBackground(cell: UITableViewCell, itemIdentifier: CustomListItemIdentifier) {
        guard let cell = cell as? DynamicBackgroundConfiguration else { return }
        cell.setAutoAdaptingBackgroundConfiguration(.mullvadListGroupedCell(), selectionType: .dimmed)
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

    private func configureLocations(_ cell: UITableViewCell, itemIdentifier: CustomListItemIdentifier) {
        var contentConfiguration = UIListContentConfiguration.mullvadValueCell(tableStyle: tableView.style)

        contentConfiguration.text = itemIdentifier.text
        cell.contentConfiguration = contentConfiguration

        if let cell = cell as? CustomCellDisclosureHandling {
            cell.disclosureType = .chevron
        }
    }
}
