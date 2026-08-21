// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

/**
 Application router delegate
 */
public protocol ApplicationRouterDelegate<RouteType>: AnyObject, Sendable {
    associatedtype RouteType: AppRouteProtocol

    /**
     Delegate should present the route and pass corresponding `Coordinator` upon completion.
     */
    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        presentWithContext context: RoutePresentationContext<RouteType>,
        animated: Bool,
        completion: @escaping @Sendable (Coordinator) -> Void
    )

    /**
     Delegate should dismiss the route.
     */
    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        dismissWithContext context: RouteDismissalContext<RouteType>,
        completion: @escaping @Sendable () -> Void
    )

    /**
     Delegate may reconsider if route presentation is still needed.

     Return `true` to proceed with presenation, otherwise `false` to prevent it.
     */
    func applicationRouter(_ router: ApplicationRouter<RouteType>, shouldPresent route: RouteType) -> Bool

    /**
     Delegate may reconsider if route dismissal should be done.

     Return `true` to proceed with dismissal, otherwise `false` to prevent it.
     */
    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        shouldDismissWithContext context: RouteDismissalContext<RouteType>
    ) -> Bool

    /**
     Delegate should handle sub-navigation for routes supporting it then call completion to tell
     router when it's done.
     */
    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        handleSubNavigationWithContext context: RouteSubnavigationContext<RouteType>,
        completion: @escaping @Sendable @MainActor () -> Void
    )
}
