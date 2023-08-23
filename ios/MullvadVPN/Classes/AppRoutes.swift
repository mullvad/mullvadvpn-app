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
 Enum type describing groups of routes. Each group is a modal layer with horizontal navigation
 inside with exception where primary navigation is a part of root controller on iPhone.
 */
enum AppRouteGroup: String, AppRouteGroupProtocol {
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
     Alert group.
     */
    case alert

    var isModal: Bool {
        switch self {
        case .primary:
            return UIDevice.current.userInterfaceIdiom == .pad

        case .selectLocation, .account, .settings, .changelog, .alert:
            return true
        }
    }

    var modalLevel: Int {
        switch self {
        case .primary:
            return 0
        case .settings, .account, .selectLocation, .changelog:
            return 1
        case .alert:
            // Alerts should always be topmost.
            return 999
        }
    }
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
     Select location route.
     */
    case selectLocation

    /**
     Changelog route.
     */
    case changelog

    /**
     Alert route.
     */
    case alert(AlertPresentation)

    /**
     Routes that are part of primary horizontal navigation group.
     */
    case tos, login, main, revoked, outOfTime, welcome

    var isExclusive: Bool {
        switch self {
        case .selectLocation, .account, .settings, .changelog, .alert:
            return true
        default:
            return false
        }
    }

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
        case .changelog:
            return .changelog
        case .selectLocation:
            return .selectLocation
        case .account:
            return .account
        case .settings:
            return .settings
        case .alert:
            return .alert
        }
    }
}
