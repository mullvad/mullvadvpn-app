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
@MainActor
public final class ApplicationRouter<RouteType: AppRouteProtocol> {
    nonisolated(unsafe) private let logger = Logger(label: "ApplicationRouter")

    private(set) var presentedRoutes: [RouteType.RouteGroupType: [PresentedRoute<RouteType>]] = [:]
    private unowned let delegate: any ApplicationRouterDelegate<RouteType>

    /// Designated initializer.
    /// Delegate object is unonwed and the caller has to guarantee that the router does not outlive it.
    public init(_ delegate: some ApplicationRouterDelegate<RouteType>) {
        self.delegate = delegate
    }

    /// Returns `true` if is the given route is currently being presented.
    public func isPresenting(route: RouteType) -> Bool {
        guard let presentedRoute = presentedRoutes[route.routeGroup] else {
            return false
        }
        return presentedRoute.contains(where: { $0.route == route })
    }

    /// Returns `true` is the given route group is currently being presented.
    public func isPresenting(group: RouteType.RouteGroupType) -> Bool {
        presentedRoutes[group] != nil
    }

    /// Presents route.
    public func present(_ route: RouteType, animated: Bool = true, metadata: Any? = nil) {
        // Pass sub-route for routes supporting sub-navigation.
        if route.supportsSubNavigation, let presentedRoute = presentedRoutes[route.routeGroup]?.first {
            presentSubRoute(route, on: presentedRoute, animated: animated)
        } else {
            presentRoute(route, animated: animated, metadata: metadata)
        }
    }

    /// Dismisses route.
    public func dismiss(route: RouteType, animated: Bool = true) {
        var routes = presentedRoutes[route.routeGroup] ?? []

        // Find the index of route to pop.
        guard let index = routes.lastIndex(where: { $0.route == route }) else {
            return
        }

        // Check if dismissing the last route.
        let isLastRoute = routes.count == 1

        let context = RouteDismissalContext(
            dismissedRoutes: [routes[index]],
            isClosing: isLastRoute,
            isAnimated: animated
        )

        // Consult with delegate whether the route should still be dismissed.
        guard delegate.applicationRouter(self, shouldDismissWithContext: context) else {
            return
        }

        if isLastRoute {
            presentedRoutes.removeValue(forKey: route.routeGroup)
        } else {
            routes.remove(at: index)
            presentedRoutes[route.routeGroup] = routes
        }

        delegate.applicationRouter(self, dismissWithContext: context) { [weak self] in
            MainActor.assumeIsolated {
                self?.logger.debug("Dismissed route: \(route)")
            }
        }
    }

    // Dismisses route group.
    public func dismiss(group: RouteType.RouteGroupType, animated: Bool = true) {
        let dismissedRoutes = presentedRoutes[group] ?? []

        guard !dismissedRoutes.isEmpty else {
            return
        }

        let context = RouteDismissalContext(
            dismissedRoutes: dismissedRoutes,
            isClosing: true,
            isAnimated: animated
        )

        // Consult with delegate whether the route should still be dismissed.
        guard delegate.applicationRouter(self, shouldDismissWithContext: context) else {
            return
        }

        presentedRoutes.removeValue(forKey: group)

        delegate.applicationRouter(self, dismissWithContext: context) { [weak self] in
            MainActor.assumeIsolated {
                self?.logger.debug("Dismissed route group: \(group)")
            }
        }
    }

    private func presentRoute(_ route: RouteType, animated: Bool, metadata: Any?) {
        // Consult with delegate whether the route should still be presented.
        if delegate.applicationRouter(self, shouldPresent: route) {
            let context = RoutePresentationContext(route: route, isAnimated: animated, metadata: metadata)

            delegate.applicationRouter(
                self,
                presentWithContext: context,
                animated: animated
            ) { [weak self] coordinator in
                MainActor.assumeIsolated {
                    if let presentable = coordinator as? Presentable {
                        presentable.onInteractiveDismissal { [weak self] coordinator in
                            MainActor.assumeIsolated {
                                self?.handleInteractiveDismissal(route: route, coordinator: coordinator)
                            }
                        }
                    }

                    let group = route.routeGroup
                    var routes = self?.presentedRoutes[group] ?? []

                    routes.append(PresentedRoute(route: route, coordinator: coordinator))
                    self?.presentedRoutes[group] = routes

                    self?.logger.debug("Presented route: \(route)")
                }
            }
        }
    }

    private func presentSubRoute(_ route: RouteType, on presentedRoute: PresentedRoute<RouteType>, animated: Bool) {
        var presentedRoute = presentedRoute

        let context = RouteSubnavigationContext(
            presentedRoute: presentedRoute,
            route: route,
            isAnimated: animated
        )

        presentedRoute.route = route
        presentedRoutes[route.routeGroup] = [presentedRoute]

        delegate.applicationRouter(self, handleSubNavigationWithContext: context) { [weak self] in
            MainActor.assumeIsolated {
                self?.logger.debug("Presented sub route: \(route)")
            }
        }
    }

    internal func handleInteractiveDismissal(route: RouteType, coordinator: Coordinator) {
        var routes = presentedRoutes[route.routeGroup] ?? []

        routes.removeAll { presentedRoute in
            presentedRoute.coordinator == coordinator
        }

        if routes.isEmpty {
            presentedRoutes.removeValue(forKey: route.routeGroup)
        } else {
            presentedRoutes[route.routeGroup] = routes
        }

        logger.debug("Dismissed route: \(route)")
    }
}
