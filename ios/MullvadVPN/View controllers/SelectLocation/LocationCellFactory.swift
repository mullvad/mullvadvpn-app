//
//  LocationCellFactory.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-17.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import UIKit

protocol LocationCellEventHandler {
    func collapseCell(for item: RelayLocation)
}

final class LocationCellFactory: CellFactoryProtocol {
    var nodeByLocation = [RelayLocation: LocationDataSource.Node]()
    var delegate: LocationCellEventHandler?
    let tableView: UITableView

    init(tableView: UITableView, nodeByLocation: [RelayLocation: LocationDataSource.Node]) {
        self.tableView = tableView
        self.nodeByLocation = nodeByLocation
    }

    func makeCell(for item: RelayLocation, indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(
            withIdentifier: LocationDataSource.CellReuseIdentifiers.locationCell.rawValue,
            for: indexPath
        )

        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    func configureCell(_ cell: UITableViewCell, item: RelayLocation, indexPath: IndexPath) {
        guard let cell = cell as? SelectLocationCell,
              let node = nodeByLocation[item] else { return }

        cell.accessibilityIdentifier = node.location.stringRepresentation
        cell.isDisabled = !node.isActive
        cell.locationLabel.text = node.displayName
        cell.showsCollapseControl = node.isCollapsible
        cell.isExpanded = node.showsChildren
        cell.didCollapseHandler = { [weak self] cell in
            self?.delegate?.collapseCell(for: item)
        }
    }
}
