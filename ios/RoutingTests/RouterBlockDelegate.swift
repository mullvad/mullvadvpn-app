//
//  RouterBlockDelegate.swift
//  RoutingTests
//
//  Created by Jon Petersson on 2023-08-22.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Routing

final class RouterBlockDelegate<RouteType: AppRouteProtocol>: ApplicationRouterDelegate, @unchecked Sendable {
    var handleRoute: ((RoutePresentationContext<RouteType>, Bool, (Coordinator) -> Void) -> Void)?
    var handleDismiss: ((RouteDismissalContext<RouteType>, () -> Void) -> Void)?
    var shouldPresent: ((RouteType) -> Bool)?
    var shouldDismiss: ((RouteDismissalContext<RouteType>) -> Bool)?
    var handleSubnavigation: (@Sendable @MainActor (RouteSubnavigationContext<RouteType>, () -> Void) -> Void)?

    nonisolated func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        presentWithContext context: RoutePresentationContext<RouteType>,
        animated: Bool,
        completion: @escaping @Sendable (Coordinator) -> Void
    ) {
        MainActor.assumeIsolated {
            handleRoute?(context, animated, completion) ?? completion(Coordinator())
        }
    }

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        dismissWithContext context: RouteDismissalContext<RouteType>,
        completion: @escaping @Sendable () -> Void
    ) {
        handleDismiss?(context, completion) ?? completion()
    }

    func applicationRouter(_ router: ApplicationRouter<RouteType>, shouldPresent route: RouteType) -> Bool {
        return shouldPresent?(route) ?? true
    }

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        shouldDismissWithContext context: RouteDismissalContext<RouteType>
    ) -> Bool {
        return shouldDismiss?(context) ?? true
    }

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        handleSubNavigationWithContext context: RouteSubnavigationContext<RouteType>,
        completion: @escaping @Sendable () -> Void
    ) {
        MainActor.assumeIsolated {
            handleSubnavigation?(context, completion) ?? completion()
        }
    }
}
