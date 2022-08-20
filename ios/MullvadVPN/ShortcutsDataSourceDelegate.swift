//
//  ShortcutsDataSourceDelegate.swift
//  MullvadVPN
//
//  Created by Nikolay Davydov on 20.08.2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

protocol ShortcutsDataSourceDelegate: AnyObject {
    func shortcutsDataSource(
        _ dataSource: ShortcutsDataSource,
        didSelectItem item: ShortcutsDataSource.Item
    )
}
