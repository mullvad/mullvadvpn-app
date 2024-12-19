//
//  CustomListDataSourceConfigurationv.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

@MainActor
class CustomListDataSourceConfiguration: NSObject {
    let dataSource: UITableViewDiffableDataSource<CustomListSectionIdentifier, CustomListItemIdentifier>
    var validationErrors: Set<CustomListFieldValidationError> = []

    var didSelectItem: ((CustomListItemIdentifier) -> Void)?

    init(dataSource: UITableViewDiffableDataSource<CustomListSectionIdentifier, CustomListItemIdentifier>) {
        self.dataSource = dataSource
    }

    func updateDataSource(
        sections: [CustomListSectionIdentifier],
        validationErrors: Set<CustomListFieldValidationError>,
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<CustomListSectionIdentifier, CustomListItemIdentifier>()

        sections.forEach { section in
            switch section {
            case .name:
                snapshot.appendSections([.name])
                snapshot.appendItems([.name], toSection: .name)
            case .addLocations:
                snapshot.appendSections([.addLocations])
                snapshot.appendItems([.addLocations], toSection: .addLocations)
            case .editLocations:
                snapshot.appendSections([.editLocations])
                snapshot.appendItems([.editLocations], toSection: .editLocations)
            case .deleteList:
                snapshot.appendSections([.deleteList])
                snapshot.appendItems([.deleteList], toSection: .deleteList)
            }
        }

        dataSource.apply(snapshot, animatingDifferences: animated)
    }

    func set(validationErrors: Set<CustomListFieldValidationError>) {
        self.validationErrors = validationErrors

        var snapshot = dataSource.snapshot()

        validationErrors.forEach { error in
            switch error {
            case .name:
                snapshot.reloadSections([.name])
            }
        }

        dataSource.apply(snapshot, animatingDifferences: false)
    }
}

extension CustomListDataSourceConfiguration: UITableViewDelegate {
    func tableView(_ tableView: UITableView, viewForHeaderInSection section: Int) -> UIView? {
        return nil
    }

    func tableView(_ tableView: UITableView, heightForHeaderInSection section: Int) -> CGFloat {
        let sectionIdentifier = dataSource.snapshot().sectionIdentifiers[section]

        return switch sectionIdentifier {
        case .name:
            16
        default:
            UITableView.automaticDimension
        }
    }

    func tableView(_ tableView: UITableView, heightForRowAt indexPath: IndexPath) -> CGFloat {
        UIMetrics.SettingsCell.customListsCellHeight
    }

    func tableView(_ tableView: UITableView, viewForFooterInSection section: Int) -> UIView? {
        let snapshot = dataSource.snapshot()

        let sectionIdentifier = snapshot.sectionIdentifiers[section]
        let itemsInSection = snapshot.itemIdentifiers(inSection: sectionIdentifier)

        let itemsWithErrors = CustomListItemIdentifier.fromFieldValidationErrors(validationErrors)
        let errorsInSection = itemsWithErrors.filter { itemsInSection.contains($0) }.compactMap { item in
            switch item {
            case .name:
                Array(validationErrors).filter { error in
                    if case .name = error {
                        return true
                    }
                    return false
                }
            case .addLocations, .editLocations, .deleteList:
                nil
            }
        }

        switch sectionIdentifier {
        case .name:
            let view = SettingsFieldValidationErrorContentView(
                configuration: SettingsFieldValidationErrorConfiguration(
                    errors: errorsInSection.flatMap { $0.settingsFieldValidationErrors }
                )
            )
            return view
        default:
            return nil
        }
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        tableView.deselectRow(at: indexPath, animated: false)

        if let item = dataSource.itemIdentifier(for: indexPath) {
            didSelectItem?(item)
        }
    }
}
