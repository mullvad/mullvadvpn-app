//
//  AppRoutes.swift
//  MullvadVPN
//
//  Created by pronebird on 17/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Routing
import UIKit

/**
 Enum type describing groups of routes.
 */
enum AppRouteGroup: AppRouteGroupProtocol {
    /**
     Primary horizontal navigation group.
     */
    case primary

    /**
     Select location group.
     */
    case selectLocation

    /**
     Account group.
     */
    case account

    /**
     Settings group.
     */
    case settings

    /**
     Changelog group.
     */
    case changelog

    /**
     Alert group. Alert id should match the id of the alert being contained.
     */
    case alert(_ alertId: String)
}

/**
 Enum type describing primary application routes.
 */
enum AppRoute: AppRouteProtocol {
    /**
     Account route.
     */
    case account

    /**
     Settings route. Contains sub-route to display.
     */
    case settings(SettingsNavigationRoute?)

    /**
     DAITA standalone route (not subsetting).
     */
    case daita

    /**
     Select location route.
     */
    case selectLocation

    /**
     Changelog standalone route (not subsetting).
     */
    case changelog

    /**
     Alert route. Alert id must be a unique string in order to produce a unique route
     that distinguishes between different kinds of alerts.
     */
    case alert(_ alertId: String)

    /**
     Routes that are part of primary horizontal navigation group.
     */
    case tos, login, main, revoked, outOfTime, welcome

    var supportsSubNavigation: Bool {
        if case .settings = self {
            return true
        } else {
            return false
        }
    }

    var routeGroup: AppRouteGroup {
        switch self {
        case .tos, .login, .main, .revoked, .outOfTime, .welcome:
            return .primary
        case .selectLocation:
            return .selectLocation
        case .account:
            return .account
        case .settings, .daita, .changelog:
            return .settings
        case let .alert(id):
            return .alert(id)
        }
    }
}
