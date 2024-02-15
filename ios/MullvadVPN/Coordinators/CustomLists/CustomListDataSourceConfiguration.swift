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
        animated: Bool,
        completion: (() -> Void)? = nil
    ) {
        var snapshot = NSDiffableDataSourceSnapshot<CustomListSectionIdentifier, CustomListItemIdentifier>()

        snapshot.appendSections([.name])
        snapshot.appendItems([.name], toSection: .name)

        snapshot.appendSections([.locations])
        snapshot.appendItems([.locations], toSection: .locations)

        dataSource.apply(snapshot, animatingDifferences: animated)
    }
}

extension CustomListDataSourceConfiguration: UITableViewDelegate {
    func tableView(_ tableView: UITableView, heightForRowAt indexPath: IndexPath) -> CGFloat {
        UIMetrics.SettingsCell.customListsCellHeight
    }

    func tableView(_ tableView: UITableView, shouldHighlightRowAt indexPath: IndexPath) -> Bool {
        guard let itemIdentifier = dataSource.itemIdentifier(for: indexPath) else { return false }
        return itemIdentifier.isSelectable
    }

    func tableView(_ tableView: UITableView, didSelectRowAt indexPath: IndexPath) {
        tableView.deselectRow(at: indexPath, animated: false)

        dataSource.itemIdentifier(for: indexPath).flatMap { item in
            didSelectItem?(item)
        }
    }
}
