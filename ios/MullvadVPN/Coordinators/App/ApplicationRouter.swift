//
//  ApplicationRouter.swift
//  MullvadVPN
//
//  Created by pronebird on 16/03/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import UIKit

/**
 Enum type describing groups of routes. Each group is a modal layer with horizontal navigation
 inside with exception where primary navigation is a part of root controller on iPhone.
 */
enum AppRouteGroup: Comparable, Equatable, Hashable {
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
     Returns `true` if group is presented modally, otherwise `false` if group is a part of root view
     controller.
     */
    var isModal: Bool {
        switch self {
        case .primary:
            return UIDevice.current.userInterfaceIdiom == .pad

        case .selectLocation, .account, .settings:
            return true
        }
    }

    private var order: Int {
        switch self {
        case .primary:
            return 0
        case .settings, .account, .selectLocation:
            return 1
        }
    }

    static func < (lhs: AppRouteGroup, rhs: AppRouteGroup) -> Bool {
        return lhs.order < rhs.order
    }
}

/**
 Enum type describing primary application routes.
 */
enum AppRoute: Equatable, Hashable {
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
     Routes that are part of primary horizontal navigation group.
     */
    case tos, changelog, login, main, revoked, outOfTime

    /**
     Returns `true` when only one route of a kind can be displayed.
     */
    var isExclusive: Bool {
        switch self {
        case .selectLocation, .account, .settings:
            return true
        default:
            return false
        }
    }

    /**
     Returns `true` if the route supports sub-navigation.
     */
    var supportsSubNavigation: Bool {
        if case .settings = self {
            return true
        } else {
            return false
        }
    }

    /**
     Navigation group to which the route belongs to.
     */
    var routeGroup: AppRouteGroup {
        switch self {
        case .tos, .changelog, .login, .main, .revoked, .outOfTime:
            return .primary
        case .selectLocation:
            return .selectLocation
        case .account:
            return .account
        case .settings:
            return .settings
        }
    }
}

/**
 Struct describing a routing request for presentation or dismissal.
 */
struct PendingRoute: Equatable {
    var operation: RouteOperation
    var animated: Bool
}

/**
 Enum type describing an attempt to fulfill the route presentation request.
 **/
enum PendingPresentationResult {
    /**
     Successfully presented the route.
     */
    case success

    /**
     The request to present this route should be dropped.
     */
    case drop

    /**
     The request to present this route cannot be fulfilled because the modal context does not allow
     for that.

     For example, on iPad, primary context cannot be presented above settings, because it enables
     access to settings by making the settings cog accessible via custom presentation controller.
     In such case the router will attempt to fulfill other requests in hope that perhaps settings
     can be dismissed first before getting back to that request.
     */
    case blockedByModalContext
}

/**
 Enum type describing an attempt to fulfill the route dismissal request.
 */
enum PendingDismissalResult {
    /**
     Successfully dismissed the route.
     */
    case success

    /**
     The request to present this route should be dropped.
     */
    case drop

    /**
     The route cannot be dismissed immediately because it's blocked by another modal presented
     above.

     The router will attempt to fulfill other requests first in hope to unblock the route by
     dismissing the modal above before getting back to that request.
     */
    case blockedByModalAbove
}

/**
 Enum describing operation over the route.
 */
enum RouteOperation: Equatable {
    /**
     Present route.
     */
    case present(AppRoute)

    /**
     Dismiss route.
     */
    case dismiss(DismissMatch)

    /**
     Returns a group of affected routes.
     */
    var routeGroup: AppRouteGroup {
        switch self {
        case let .present(route):
            return route.routeGroup
        case let .dismiss(dismissMatch):
            return dismissMatch.routeGroup
        }
    }
}

/**
 Enum type describing a single route or a group of routes requested to be dismissed.
 */
enum DismissMatch: Equatable {
    case group(AppRouteGroup)
    case singleRoute(AppRoute)

    /**
     Returns a group of affected routes.
     */
    var routeGroup: AppRouteGroup {
        switch self {
        case let .group(group):
            return group
        case let .singleRoute(route):
            return route.routeGroup
        }
    }
}

/**
 Struct describing presented route.
 */
struct PresentedRoute: Equatable {
    var route: AppRoute
    var coordinator: Coordinator
}

/**
 Struct holding information used by delegate to perform dismissal of the route(s) in subject.
 */
struct RouteDismissalContext {
    /**
     Specific routes that are being dismissed.
     */
    var dismissedRoutes: [PresentedRoute]

    /**
     Whether the entire group is being dismissed.
     */
    var isClosing: Bool

    /**
     Whether transition is animated.
     */
    var isAnimated: Bool
}

/**
 Struct holding information used by delegate to perform sub-navigation of the route in subject.
 */
struct RouteSubnavigationContext {
    var presentedRoute: PresentedRoute
    var route: AppRoute
    var isAnimated: Bool
}

/**
 Application router delegate
 */
protocol ApplicationRouterDelegate: AnyObject {
    /**
     Delegate should present the route and pass corresponding `Coordinator` upon completion.
     */
    func applicationRouter(
        _ router: ApplicationRouter,
        route: AppRoute,
        animated: Bool,
        completion: @escaping (Coordinator) -> Void
    )

    /**
     Delegate should dismiss the route.
     */
    func applicationRouter(
        _ router: ApplicationRouter,
        dismissWithContext context: RouteDismissalContext,
        completion: @escaping () -> Void
    )

    /**
     Delegate may reconsider if route presentation is still needed.

     Return `true` to proceed with presenation, otherwise `false` to prevent it.
     */
    func applicationRouter(_ router: ApplicationRouter, shouldPresent route: AppRoute) -> Bool

    /**
     Delegate may reconsider if route dismissal should be done.

     Return `true` to proceed with dismissal, otherwise `false` to prevent it.
     */
    func applicationRouter(
        _ router: ApplicationRouter,
        shouldDismissWithContext context: RouteDismissalContext
    ) -> Bool

    /**
     Delegate should handle sub-navigation for routes supporting it then call completion to tell
     router when it's done.
     */
    func applicationRouter(
        _ router: ApplicationRouter,
        handleSubNavigationWithContext context: RouteSubnavigationContext,
        completion: @escaping () -> Void
    )
}

/**
 Main application router.
 */
final class ApplicationRouter {
    private let logger = Logger(label: "ApplicationRouter")

    private(set) var modalStack: [AppRouteGroup] = []
    private var presentedRoutes: [AppRouteGroup: [PresentedRoute]] = [:]

    private var pendingRoutes = [PendingRoute]()
    private var isProcessingPendingRoutes = false

    private unowned let delegate: ApplicationRouterDelegate

    /**
     Designated initializer.

     Delegate object is unonwed and the caller has to guarantee that the router does not outlive it.
     */
    init(_ delegate: ApplicationRouterDelegate) {
        self.delegate = delegate
    }

    /**
     Returns `true` is the given route group is currently being presented.
     */
    func isPresenting(_ group: AppRouteGroup) -> Bool {
        return modalStack.contains(group)
    }

    /**
     Enqueue route for presetnation.
     */
    func present(_ route: AppRoute, animated: Bool = true) {
        enqueue(PendingRoute(
            operation: .present(route),
            animated: animated
        ))
    }

    /**
     Enqueue dismissal of the route.
     */
    func dismiss(_ route: AppRoute, animated: Bool = true) {
        enqueue(PendingRoute(
            operation: .dismiss(.singleRoute(route)),
            animated: animated
        ))
    }

    /**
     Enqueue dismissal of a group of routes.
     */
    func dismissAll(_ group: AppRouteGroup, animated: Bool = true) {
        enqueue(PendingRoute(
            operation: .dismiss(.group(group)),
            animated: animated
        ))
    }

    private func enqueue(_ pendingRoute: PendingRoute) {
        logger.debug("Enqueue \(pendingRoute.operation).")

        pendingRoutes.append(pendingRoute)

        if !isProcessingPendingRoutes {
            processPendingRoutes()
        }
    }

    private func presentRoute(
        _ route: AppRoute,
        animated: Bool,
        completion: @escaping (PendingPresentationResult) -> Void
    ) {
        /**
         Pass sub-route for routes supporting sub-navigation.
         */
        if route.supportsSubNavigation, modalStack.contains(route.routeGroup),
           var presentedRoute = presentedRoutes[route.routeGroup]?.first
        {
            let context = RouteSubnavigationContext(
                presentedRoute: presentedRoute,
                route: route,
                isAnimated: animated
            )

            presentedRoute.route = route
            presentedRoutes[route.routeGroup] = [presentedRoute]

            delegate.applicationRouter(self, handleSubNavigationWithContext: context) {
                completion(.success)
            }

            return
        }

        /**
         Drop duplicate routes.
         */
        if route.isExclusive, modalStack.contains(route.routeGroup) {
            completion(.drop)
            return
        }

        /**
         Drop if the last presented route within the group is the same.
         */
        if !route.isExclusive, presentedRoutes[route.routeGroup]?.last?.route == route {
            completion(.drop)
            return
        }

        /**
         Check if route can be presented above the last route in the modal stack.
         */
        if let lastRouteGroup = modalStack.last, lastRouteGroup >= route.routeGroup,
           route.routeGroup.isModal, route.isExclusive
        {
            completion(.blockedByModalContext)
            return
        }

        /**
         Consult with delegate whether the route should still be presented.
         */
        if delegate.applicationRouter(self, shouldPresent: route) {
            delegate.applicationRouter(self, route: route, animated: animated) { coordinator in
                /*
                 Synchronize router when modal controllers are removed by swipe.
                 */
                if let presentable = coordinator as? Presentable {
                    presentable.onInteractiveDismissal { [weak self] coordinator in
                        self?.handleInteractiveDismissal(route: route, coordinator: coordinator)
                    }
                }

                self.addPresentedRoute(PresentedRoute(route: route, coordinator: coordinator))

                completion(.success)
            }
        } else {
            completion(.drop)
        }
    }

    private func dismissGroup(
        _ dismissGroup: AppRouteGroup,
        animated: Bool,
        completion: @escaping (PendingDismissalResult) -> Void
    ) {
        /**
         Check if routes corresponding to the group requested for dismissal are present.
         */
        guard modalStack.contains(dismissGroup) else {
            completion(.drop)
            return
        }

        /**
         Check if the group can be dismissed and it's not blocked by another group presented above.
         */
        if modalStack.last != dismissGroup, dismissGroup.isModal {
            completion(.blockedByModalAbove)
            return
        }

        let dismissedRoutes = presentedRoutes[dismissGroup] ?? []
        assert(!dismissedRoutes.isEmpty)

        let context = RouteDismissalContext(
            dismissedRoutes: dismissedRoutes,
            isClosing: true,
            isAnimated: animated
        )

        /**
         Consult with delegate whether the route should still be dismissed.
         */
        guard delegate.applicationRouter(self, shouldDismissWithContext: context) else {
            completion(.drop)
            return
        }

        presentedRoutes.removeValue(forKey: dismissGroup)
        modalStack.removeAll { $0 == dismissGroup }

        delegate.applicationRouter(self, dismissWithContext: context) {
            completion(.success)
        }
    }

    private func dismissRoute(
        _ dismissRoute: AppRoute,
        animated: Bool,
        completion: @escaping (PendingDismissalResult) -> Void
    ) {
        var routes = presentedRoutes[dismissRoute.routeGroup] ?? []

        // Find the index of route to pop.
        guard let index = routes.lastIndex(where: { $0.route == dismissRoute }) else {
            completion(.drop)
            return
        }

        // Check if dismissing the last route in horizontal navigation group.
        let isLastRoute = routes.count == 1

        // Check if the route can be dismissed and there is no other modal above.
        if let lastModalGroup = modalStack.last,
           lastModalGroup != dismissRoute.routeGroup,
           dismissRoute.routeGroup.isModal,
           isLastRoute
        {
            completion(.blockedByModalAbove)
            return
        }

        let context = RouteDismissalContext(
            dismissedRoutes: [routes[index]],
            isClosing: isLastRoute,
            isAnimated: animated
        )

        /**
         Consult with delegate whether the route should still be dismissed.
         */
        guard delegate.applicationRouter(self, shouldDismissWithContext: context) else {
            completion(.drop)
            return
        }

        if isLastRoute {
            presentedRoutes.removeValue(forKey: dismissRoute.routeGroup)
            modalStack.removeAll { $0 == dismissRoute.routeGroup }
        } else {
            routes.remove(at: index)
            presentedRoutes[dismissRoute.routeGroup] = routes
        }

        delegate.applicationRouter(self, dismissWithContext: context) {
            completion(.success)
        }
    }

    private func processPendingRoutes(skipRouteGroups: Set<AppRouteGroup> = []) {
        isProcessingPendingRoutes = true

        let pendingRoute = pendingRoutes.first { pendingRoute in
            return !skipRouteGroups.contains(pendingRoute.operation.routeGroup)
        }

        guard let pendingRoute = pendingRoute else {
            isProcessingPendingRoutes = false
            return
        }

        switch pendingRoute.operation {
        case let .present(route):
            presentRoute(route, animated: pendingRoute.animated) { result in
                switch result {
                case .success, .drop:
                    self.finishPendingRoute(pendingRoute)

                case .blockedByModalContext:
                    /**
                     Present next route if this one is not ready to be presented.
                     */
                    self.processPendingRoutes(
                        skipRouteGroups: skipRouteGroups.union([route.routeGroup])
                    )
                }
            }

        case let .dismiss(dismissMatch):
            handleDismissal(dismissMatch, animated: pendingRoute.animated) { result in
                switch result {
                case .success, .drop:
                    self.finishPendingRoute(pendingRoute)

                case .blockedByModalAbove:
                    /**
                     If router cannot dismiss modal because there is one above,
                     try walking down the queue and see if there is a dismissal request that could
                     resolve that.
                     */
                    self.processPendingRoutes(
                        skipRouteGroups: skipRouteGroups.union([dismissMatch.routeGroup])
                    )
                }
            }
        }
    }

    private func handleDismissal(
        _ dismissMatch: DismissMatch,
        animated: Bool,
        completion: @escaping (PendingDismissalResult) -> Void
    ) {
        switch dismissMatch {
        case let .singleRoute(route):
            dismissRoute(route, animated: animated, completion: completion)

        case let .group(group):
            dismissGroup(group, animated: animated, completion: completion)
        }
    }

    private func finishPendingRoute(_ pendingRoute: PendingRoute) {
        if let index = pendingRoutes.firstIndex(of: pendingRoute) {
            pendingRoutes.remove(at: index)
        }

        processPendingRoutes()
    }

    private func handleInteractiveDismissal(route: AppRoute, coordinator: Coordinator) {
        var routes = presentedRoutes[route.routeGroup] ?? []

        routes.removeAll { presentedRoute in
            return presentedRoute.coordinator == coordinator
        }

        if routes.isEmpty {
            presentedRoutes.removeValue(forKey: route.routeGroup)
            modalStack.removeAll { $0 == route.routeGroup }
        } else {
            presentedRoutes[route.routeGroup] = routes
        }

        if !isProcessingPendingRoutes {
            processPendingRoutes()
        }
    }

    private func addPresentedRoute(_ presented: PresentedRoute) {
        let group = presented.route.routeGroup
        var routes = presentedRoutes[group] ?? []

        if presented.route.isExclusive {
            routes = [presented]
        } else {
            routes.append(presented)
        }

        presentedRoutes[group] = routes

        if !modalStack.contains(group) {
            if group.isModal {
                modalStack.append(group)
            } else {
                modalStack.insert(group, at: 0)
            }
        }
    }
}
