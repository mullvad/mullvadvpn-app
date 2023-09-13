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
 Main application router.
 */
public final class ApplicationRouter<RouteType: AppRouteProtocol> {
    private let logger = Logger(label: "ApplicationRouter")

    private(set) var modalStack: [RouteType.RouteGroupType] = []
    private(set) var presentedRoutes: [RouteType.RouteGroupType: [PresentedRoute<RouteType>]] = [:]

    private var pendingRoutes = [PendingRoute<RouteType>]()
    private var isProcessingPendingRoutes = false

    private unowned let delegate: any ApplicationRouterDelegate<RouteType>

    /**
     Designated initializer.

     Delegate object is unonwed and the caller has to guarantee that the router does not outlive it.
     */
    public init(_ delegate: some ApplicationRouterDelegate<RouteType>) {
        self.delegate = delegate
    }

    /**
     Returns `true` is the given route group is currently being presented.
     */
    public func isPresenting(group: RouteType.RouteGroupType) -> Bool {
        modalStack.contains(group)
    }

    /**
     Returns `true` if is the given route  is currently being presented.
     */
    public func isPresenting(route: RouteType) -> Bool {
        guard let presentedRoute = presentedRoutes[route.routeGroup] else {
            return false
        }
        return presentedRoute.contains(where: { $0.route == route })
    }

    /**
     Enqueue route for presetnation.
     */
    public func present(_ route: RouteType, animated: Bool = true, metadata: Any? = nil) {
        enqueue(PendingRoute(
            operation: .present(route),
            animated: animated,
            metadata: metadata
        ))
    }

    /**
     Enqueue dismissal of the route.
     */
    public func dismiss(_ route: RouteType, animated: Bool = true) {
        enqueue(PendingRoute(
            operation: .dismiss(.singleRoute(route)),
            animated: animated
        ))
    }

    /**
     Enqueue dismissal of a group of routes.
     */
    public func dismissAll(_ group: RouteType.RouteGroupType, animated: Bool = true) {
        enqueue(PendingRoute(
            operation: .dismiss(.group(group)),
            animated: animated
        ))
    }

    private func enqueue(_ pendingRoute: PendingRoute<RouteType>) {
        logger.debug("\(pendingRoute.operation).")

        pendingRoutes.append(pendingRoute)

        if !isProcessingPendingRoutes {
            processPendingRoutes()
        }
    }

    private func presentRoute(
        _ route: RouteType,
        animated: Bool,
        metadata: Any?,
        completion: @escaping (PendingPresentationResult) -> Void
    ) {
        /**
         Pass sub-route for routes supporting sub-navigation.
         */
        if route.supportsSubNavigation, modalStack.contains(route.routeGroup),
           var presentedRoute = presentedRoutes[route.routeGroup]?.first {
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
         Drop duplicate exclusive routes.
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
        if let
            // Get current modal route.
            lastRouteGroup = modalStack.last,
            // Check if incoming route is modal.
            route.routeGroup.isModal,
            // Check whether incoming route can be presented on top of current.
            (lastRouteGroup > route.routeGroup) ||
            // OR, check whether incoming exclusive route can be presented on top of current.
            (lastRouteGroup >= route.routeGroup && route.isExclusive) {
            completion(.blockedByModalContext)
            return
        }

        /**
         Consult with delegate whether the route should still be presented.
         */
        if delegate.applicationRouter(self, shouldPresent: route) {
            let context = RoutePresentationContext(route: route, isAnimated: animated, metadata: metadata)

            delegate.applicationRouter(self, presentWithContext: context, animated: animated) { coordinator in
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
        _ dismissGroup: RouteType.RouteGroupType,
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
        _ dismissRoute: RouteType,
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
           isLastRoute {
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

    private func processPendingRoutes(skipRouteGroups: Set<RouteType.RouteGroupType> = []) {
        isProcessingPendingRoutes = true

        let pendingRoute = pendingRoutes.first { pendingRoute in
            !skipRouteGroups.contains(pendingRoute.operation.routeGroup)
        }

        guard let pendingRoute else {
            isProcessingPendingRoutes = false
            return
        }

        switch pendingRoute.operation {
        case let .present(route):
            presentRoute(route, animated: pendingRoute.animated, metadata: pendingRoute.metadata) { result in
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
        _ dismissMatch: DismissMatch<RouteType>,
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

    private func finishPendingRoute(_ pendingRoute: PendingRoute<RouteType>) {
        if let index = pendingRoutes.firstIndex(of: pendingRoute) {
            pendingRoutes.remove(at: index)
        }

        processPendingRoutes()
    }

    private func handleInteractiveDismissal(route: RouteType, coordinator: Coordinator) {
        var routes = presentedRoutes[route.routeGroup] ?? []

        routes.removeAll { presentedRoute in
            presentedRoute.coordinator == coordinator
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

    private func addPresentedRoute(_ presented: PresentedRoute<RouteType>) {
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
