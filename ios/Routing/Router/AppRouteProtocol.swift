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
public protocol AppRouteGroupProtocol: Comparable, Equatable, Hashable, Sendable {}

/**
 Formal protocol describing a single route.
 */
public protocol AppRouteProtocol: Equatable, Hashable, Sendable {
    associatedtype RouteGroupType: AppRouteGroupProtocol

    /**
     Returns `true` if the route supports sub-navigation.
     */
    var supportsSubNavigation: Bool { get }

    /**
     Navigation group to which the route belongs to.
     */
    var routeGroup: RouteGroupType { get }
}
