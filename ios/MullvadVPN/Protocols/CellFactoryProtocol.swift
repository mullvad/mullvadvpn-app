// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import UIKit

/// Protocol for creating factories to make ``UITableViewCell``s of various kinds.
/// Typically used in conjunction with a ``UITableViewDiffableDataSource.CellProvider``.
protocol CellFactoryProtocol {
    associatedtype ItemIdentifier

    var tableView: UITableView { get }

    func makeCell(for item: ItemIdentifier, indexPath: IndexPath) -> UITableViewCell
    func configureCell(_ cell: UITableViewCell, item: ItemIdentifier, indexPath: IndexPath)
}
