//
//  RoutingTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 14/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import Routing
import XCTest

final class RoutingTests: XCTestCase {
    override func setUpWithError() throws {}
    override func tearDownWithError() throws {}

    private func createDelegate<T: AppRouteProtocol>(shouldPresent: Bool = true) -> RouterBlockDelegate<T> {
        let delegate = RouterBlockDelegate<T>()

        delegate.handleRoute = { _, _, completion in completion(Coordinator()) }
        delegate.shouldPresent = { _ in shouldPresent }

        return delegate
    }
}

// MARK: Horisontal flow tests

extension RoutingTests {
    func testPresentHorisontalRoute() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var isExclusive: Bool { false }
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horisontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)

        XCTAssertTrue(router.isPresenting(route: .one))
    }

    func testShouldDropHorisontalRoute() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var isExclusive: Bool { false }
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horisontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate(shouldPresent: false)
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)

        XCTAssertFalse(router.isPresenting(route: .one))
    }

    func testShouldDropIdenticalHorisontalRouteInSequence() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var isExclusive: Bool { false }
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horisontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)
        router.present(.one)

        XCTAssertEqual(router.presentedRoutes[.horisontal]?.count, 1)
    }
}

// MARK: Modal flow tests

extension RoutingTests {
    func testPresentModalRoutesOfDifferentLevels() throws {
        enum TestRoute: AppRouteProtocol {
            case one, two
            var isExclusive: Bool { false }
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { self == .one ? .modalOne : .modalTwo }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)
        router.present(.two)

        XCTAssertEqual(router.modalStack.count, 2)
    }

    func testPresentModalRoutesOfDifferentLevelsInWrongOrder() throws {
        enum TestRoute: AppRouteProtocol {
            case one, two
            var isExclusive: Bool { false }
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { self == .one ? .modalOne : .modalTwo }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.two)
        router.present(.one)

        XCTAssertTrue(router.isPresenting(route: .two))
        XCTAssertEqual(router.modalStack.count, 1)
    }

    func testShouldDropSameLevelModalRouteIfPreceededByExclusve() throws {
        enum TestRoute: AppRouteProtocol {
            case one, two
            var isExclusive: Bool { self == .one }
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .modalOne }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)
        router.present(.two)

        XCTAssertTrue(router.isPresenting(route: .one))
        XCTAssertEqual(router.modalStack.count, 1)
    }
}

enum TestRouteGroup: AppRouteGroupProtocol {
    case horisontal, modalOne, modalTwo

    var isModal: Bool {
        switch self {
        case .horisontal:
            return false
        case .modalOne, .modalTwo:
            return true
        }
    }

    var modalLevel: Int {
        switch self {
        case .horisontal:
            return 0
        case .modalOne:
            return 1
        case .modalTwo:
            return 2
        }
    }
}

class RouterBlockDelegate<RouteType: AppRouteProtocol>: ApplicationRouterDelegate {
    var handleRoute: ((RouteType, Bool, (Coordinator) -> Void) -> Void)?
    var handleDismiss: ((RouteDismissalContext<RouteType>, () -> Void) -> Void)?
    var shouldPresent: ((RouteType) -> Bool)?
    var shouldDismiss: ((RouteDismissalContext<RouteType>) -> Bool)?
    var handleSubnavigation: ((RouteSubnavigationContext<RouteType>, () -> Void) -> Void)?

    func applicationRouter(
        _ router: ApplicationRouter<RouteType>,
        route: RouteType,
        animated: Bool,
        completion: @escaping (Coordinator) -> Void
    ) {
        handleRoute?(route, animated, completion) ?? completion(Coordinator())
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
