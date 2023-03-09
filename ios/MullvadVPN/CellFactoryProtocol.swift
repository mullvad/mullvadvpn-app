//
//  CellFactoryProtocol.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/// Protocol for creating factories to make ``UITableViewCell``s of various kinds.
/// Typically used in conjunction with a ``UITableViewDiffableDataSource.CellProvider``.
protocol CellFactoryProtocol {
    associatedtype ItemIdentifier

    var tableView: UITableView? { get set }

    func makeCell(for item: ItemIdentifier, indexPath: IndexPath) -> UITableViewCell
}
