//
//  CustomListDataSourceConfigurationv.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2024-02-14.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

import UIKit

class CustomListDataSourceConfiguration: NSObject {
    let dataSource: UITableViewDiffableDataSource<CustomListSectionIdentifier, CustomListItemIdentifier>

    var didSelectItem: ((CustomListItemIdentifier) -> Void)?

    init(dataSource: UITableViewDiffableDataSource<CustomListSectionIdentifier, CustomListItemIdentifier>) {
        self.dataSource = dataSource
    }

    func updateDataSource(
        sections: [CustomListSectionIdentifier],
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
}

extension CustomListDataSourceConfiguration: UITableViewDelegate {
    func tableView(_ tableView: UITableView, heightForRowAt indexPath: IndexPath) -> CGFloat {
        UIMetrics.SettingsCell.customListsCellHeight
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        tableView.deselectRow(at: indexPath, animated: false)

        dataSource.itemIdentifier(for: indexPath).flatMap { item in
            didSelectItem?(item)
        }
    }
}
