//
//  RelayFilterCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-06-02.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension RelayFilterSelection {
    @MainActor
    struct CellFactory: @preconcurrency CellFactoryProtocol {
        let tableView: UITableView

        func makeCell(for item: DataSource.Item, indexPath: IndexPath) -> UITableViewCell {
            let cell = tableView.dequeueReusableCell(
                withIdentifier: DataSource.CellReuseIdentifiers.allCases[indexPath.section].rawValue,
                for: indexPath
            )
            configureCell(cell, item: item, indexPath: indexPath)

            return cell
        }

        func configureCell(
            _ cell: UITableViewCell,
            item: DataSource.Item,
            indexPath: IndexPath
        ) {
            switch item.type {
            case .ownershipAny, .ownershipOwned, .ownershipRented:
                configureOwnershipCell(cell as? SelectableSettingsCell, item: item)
            case .allProviders, .provider:
                configureProviderCell(cell as? CheckableSettingsCell, item: item)
            }
        }

        private func configureOwnershipCell(_ cell: SelectableSettingsCell?, item: DataSource.Item) {
            guard let cell = cell else { return }

            cell.titleLabel.text = item.name

            let accessibilityIdentifier: AccessibilityIdentifier
            switch item.type {
            case .ownershipAny:
                accessibilityIdentifier = .ownershipAnyCell
            case .ownershipOwned:
                accessibilityIdentifier = .ownershipMullvadOwnedCell
            case .ownershipRented:
                accessibilityIdentifier = .ownershipRentedCell
            default:
                assertionFailure("Unexpected ownership item: \(item)")
                return
            }

            cell.setAccessibilityIdentifier(accessibilityIdentifier)
            cell.applySubCellStyling()
        }

        private func configureProviderCell(_ cell: CheckableSettingsCell?, item: DataSource.Item) {
            guard let cell = cell else { return }
            let alpha = item.isEnabled ? 1.0 : 0.2

            cell.titleLabel.text = item.name
            cell.detailTitleLabel.text = item.description

            if item.type == .allProviders {
                setFontWeight(.semibold, to: cell.titleLabel)
            } else {
                setFontWeight(.regular, to: cell.titleLabel)
            }

            cell.applySubCellStyling()
            cell.setAccessibilityIdentifier(.relayFilterProviderCell)
            cell.titleLabel.alpha = alpha
            cell.detailTitleLabel.alpha = alpha
        }

        private func setFontWeight(_ weight: UIFont.Weight, to label: UILabel) {
            label.font = UIFont.systemFont(ofSize: label.font.pointSize, weight: weight)
        }
    }
}
