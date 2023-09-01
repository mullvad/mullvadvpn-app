//
//  RouterBlockDelegate.swift
//  RoutingTests
//
//  Created by Jon Petersson on 2023-08-22.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Routing

class RouterBlockDelegate<RouteType: AppRouteProtocol>: ApplicationRouterDelegate {
    var handleRoute: ((RoutePresentationContext<RouteType>, Bool, (Coordinator) -> Void) -> Void)?
    var handleDismiss: ((RouteDismissalContext<RouteType>, () -> Void) -> Void)?
    var shouldPresent: ((RouteType) -> Bool)?
    var shouldDismiss: ((RouteDismissalContext<RouteType>) -> Bool)?
    var handleSubnavigation: ((RouteSubnavigationContext<RouteType>, () -> Void) -> Void)?

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        presentWithContext context: RoutePresentationContext<RouteType>,
        animated: Bool,
        completion: @escaping (Coordinator) -> Void
    ) {
        handleRoute?(context, animated, completion) ?? completion(Coordinator())
    }

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        dismissWithContext context: RouteDismissalContext<RouteType>,
        completion: @escaping () -> Void
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
        completion: @escaping () -> Void
    ) {
        handleSubnavigation?(context, completion) ?? completion()
    }
}
