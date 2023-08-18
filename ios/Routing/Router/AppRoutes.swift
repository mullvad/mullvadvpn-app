//
//  AppRoutes.swift
//  MullvadVPN
//
//  Created by pronebird on 17/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import UIKit

/**
 Enum type describing groups of routes. Each group is a modal layer with horizontal navigation
 inside with exception where primary navigation is a part of root controller on iPhone.
 */
public enum AppRouteGroup: AppRouteGroupProtocol {
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

    public var isModal: Bool {
        switch self {
        case .primary:
            return UIDevice.current.userInterfaceIdiom == .pad

        case .selectLocation, .account, .settings, .changelog:
            return true
        }
    }

    public var modalLevel: Int {
        switch self {
        case .primary:
            return 0
        case .settings, .account, .selectLocation, .changelog:
            return 1
        }
    }
}

/**
 Enum type describing primary application routes.
 */
public enum AppRoute: AppRouteProtocol {
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
     Routes that are part of primary horizontal navigation group.
     */
    case tos, login, main, revoked, outOfTime, welcome, setupAccountCompleted

    public var isExclusive: Bool {
        switch self {
        case .selectLocation, .account, .settings, .changelog:
            return true
        default:
            return false
        }
    }

    public var supportsSubNavigation: Bool {
        if case .settings = self {
            return true
        } else {
            return false
        }
    }

    public var routeGroup: AppRouteGroup {
        switch self {
        case .tos, .login, .main, .revoked, .outOfTime, .welcome, .setupAccountCompleted:
            return .primary
        case .changelog:
            return .changelog
        case .selectLocation:
            return .selectLocation
        case .account:
            return .account
        case .settings:
            return .settings
        }
    }
}

/// Enum type describing sub-routes for Settings functionality.
public enum SettingsNavigationRoute: Equatable {
    case root
    case preferences
    case problemReport
    case faq
}
