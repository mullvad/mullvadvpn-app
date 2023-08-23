//
//  AppRouteProtocol.swift
//  MullvadVPN
//
//  Created by pronebird on 17/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation

/**
 Formal protocol describing a group of routes.
 */
public protocol AppRouteGroupProtocol: Comparable, Equatable, Hashable {
    /**
     Returns `true` if group is presented modally, otherwise `false` if group is a part of root view
     controller.
     */
    var isModal: Bool { get }

    /**
     Defines a modal level that's used for restricting modal presentation.

     A group with higher modal level can be presented above a group with lower level but not the other way around. For example, if a modal group is represented by
     `UIAlertController`, it should have the highest level (i.e `Int.max`) to prevent other modals from being presented above it, however it enables alert
     controller to be presented above any other modal.
     */
    var modalLevel: Int { get }
}

/**
 Default implementation of `Comparable` for `AppRouteGroupProtocol` which compares `modalLevel` of both sides.
 */
extension AppRouteGroupProtocol {
    public static func < (lhs: Self, rhs: Self) -> Bool {
        lhs.modalLevel < rhs.modalLevel
    }

    public static func <= (lhs: Self, rhs: Self) -> Bool {
        lhs.modalLevel <= rhs.modalLevel
    }
}

/**
 Formal protocol describing a single route.
 */
public protocol AppRouteProtocol: Equatable, Hashable {
    associatedtype RouteGroupType: AppRouteGroupProtocol

    /**
     Returns `true` when only one route of a kind can be displayed.
     */
    var isExclusive: Bool { get }

    /**
     Returns `true` if the route supports sub-navigation.
     */
    var supportsSubNavigation: Bool { get }

    /**
     Navigation group to which the route belongs to.
     */
    var routeGroup: RouteGroupType { get }
}
