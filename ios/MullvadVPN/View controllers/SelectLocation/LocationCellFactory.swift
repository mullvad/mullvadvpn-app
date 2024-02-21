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
    func toggleCell(for item: LocationCellViewModel)
}

final class LocationCellFactory: CellFactoryProtocol {
    var delegate: LocationCellEventHandler?
    let tableView: UITableView
    let reuseIdentifier: String

    init(
        tableView: UITableView,
        reuseIdentifier: String
    ) {
        self.tableView = tableView
        self.reuseIdentifier = reuseIdentifier
    }

    func makeCell(for item: LocationCellViewModel, indexPath: IndexPath) -> UITableViewCell {
        let cell = tableView.dequeueReusableCell(
            withIdentifier: reuseIdentifier,
            for: indexPath
        )

        configureCell(cell, item: item, indexPath: indexPath)

        return cell
    }

    func configureCell(_ cell: UITableViewCell, item: LocationCellViewModel, indexPath: IndexPath) {
        guard let cell = cell as? LocationCell else { return }

        cell.accessibilityIdentifier = item.node.nodeName
        cell.locationLabel.text = item.node.nodeName
        cell.showsCollapseControl = !item.node.children.isEmpty
        cell.isExpanded = item.node.showsChildren
        cell.didCollapseHandler = { [weak self] _ in
            self?.delegate?.toggleCell(for: item)
        }
    }
}
