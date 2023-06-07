//
//  RelayFilterCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-02.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

struct RelayFilterCellFactory: CellFactoryProtocol {
    private enum OwnershipString: String {
        case any = "Any"
        case owned = "Mullvad owned only"
        case rented = "Rented only"
    }

    private enum ProviderString: String {
        case all = "All providers"
    }

    let tableView: UITableView

    init(tableView: UITableView) {
        self.tableView = tableView
    }

    func makeCell(for item: RelayFilterDataSource.Item, indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(withIdentifier: item.reuseIdentifier.rawValue, for: indexPath)
        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    func configureCell(_ cell: UITableViewCell, item: RelayFilterDataSource.Item, indexPath: IndexPath) {
        switch item {
        case .ownershipAny, .ownershipOwned, .ownershipRented:
            configureOwnershipCell(cell, item: item)
        case .allProviders, .provider:
            configureProviderCell(cell, item: item)
        }
    }

    private func configureOwnershipCell(_ cell: UITableViewCell, item: RelayFilterDataSource.Item) {
        guard let cell = cell as? SelectableSettingsCell else { return }

        var title = ""
        switch item {
        case .ownershipAny:
            title = OwnershipString.any.rawValue
        case .ownershipOwned:
            title = OwnershipString.owned.rawValue
        case .ownershipRented:
            title = OwnershipString.rented.rawValue
        default:
            assertionFailure("Item mismatch. Got: \(item)")
        }

        cell.titleLabel.text = NSLocalizedString(
            "RELAY_FILTER_CELL_LABEL",
            tableName: "Relay filter ownership cell",
            value: title,
            comment: ""
        )

        cell.applySubCellStyling()
        cell.accessibilityIdentifier = "RelayFilterOwnershipCell"
    }

    private func configureProviderCell(_ cell: UITableViewCell, item: RelayFilterDataSource.Item) {
        guard let cell = cell as? CheckableSettingsCell else { return }

        var title = ""
        switch item {
        case .allProviders:
            title = ProviderString.all.rawValue
            setFontWeight(.semibold, to: cell.titleLabel)
        case let .provider(name):
            title = name
            setFontWeight(.regular, to: cell.titleLabel)
        default:
            assertionFailure("Item mismatch. Got: \(item)")
        }

        cell.titleLabel.text = NSLocalizedString(
            "RELAY_FILTER_CELL_LABEL",
            tableName: "Relay filter provider cell",
            value: title,
            comment: ""
        )

        cell.applySubCellStyling()
        cell.accessibilityIdentifier = "RelayFilterProviderCell"
    }

    private func setFontWeight(_ weight: UIFont.Weight, to label: UILabel) {
        label.font = UIFont.systemFont(ofSize: label.font.pointSize, weight: .semibold)
    }
}
