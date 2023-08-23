//
//  Coordinator+Router.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2023-08-24.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Routing

extension Coordinator {
    var applicationRouter: ApplicationRouter<AppRoute>? {
        var appCoordinator: Coordinator? = self

        while appCoordinator?.parent != nil {
            appCoordinator = appCoordinator?.parent
        }

        return (appCoordinator as? ApplicationCoordinator)?.router
    }
}
