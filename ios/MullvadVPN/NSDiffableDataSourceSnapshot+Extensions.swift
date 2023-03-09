//
//  NSDiffableDataSourceSnapshot+Extensions.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-03-09.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension NSDiffableDataSourceSnapshot {
    func itemForIndexPath(_ indexPath: IndexPath) -> ItemIdentifierType? {
        guard indexPath.section < sectionIdentifiers.count else { return nil }

        let sectionObject = sectionIdentifiers[indexPath.section]
        let itemObjects = itemIdentifiers(inSection: sectionObject)

        if indexPath.row < itemObjects.count {
            return itemObjects[indexPath.row]
        } else {
            return nil
        }
    }
}
