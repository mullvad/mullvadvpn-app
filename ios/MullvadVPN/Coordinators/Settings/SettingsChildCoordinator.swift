//
//  SettingsChildCoordinator.swift
//  MullvadVPN
//
//  Created by pronebird on 08/11/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Routing

/// Types that are child coordinators of ``SettingsCoordinator``.
protocol SettingsChildCoordinator: Coordinator {
    func start(animated: Bool)
}
