//
//  RoutingTests.swift
//  MullvadVPNTests
//
//  Created by pronebird on 14/08/2023.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

@testable import Routing
import XCTest

@MainActor
final class RoutingTests: XCTestCase {
    private func createDelegate<T: AppRouteProtocol>(shouldPresent: Bool = true) -> RouterBlockDelegate<T> {
        let delegate = RouterBlockDelegate<T>()

        delegate.handleRoute = { _, _, completion in completion(Coordinator()) }
        delegate.shouldPresent = { _ in shouldPresent }

        return delegate
    }

    func testPresentRoute() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horizontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)

        XCTAssertTrue(router.isPresenting(route: .one))
    }

    func testPresentSubRoute() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            case two
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horizontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)
        router.present(.two)

        XCTAssertTrue(router.isPresenting(route: .one))
        XCTAssertTrue(router.isPresenting(route: .two))
    }

    func testPresentGroup() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horizontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)
        XCTAssertTrue(router.isPresenting(group: .horizontal))
    }

    func testShouldDropRoute() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horizontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate(shouldPresent: false)
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)

        XCTAssertFalse(router.isPresenting(route: .one))
    }

    func testDismissRoute() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horizontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)
        router.dismiss(route: .one)

        XCTAssertFalse(router.isPresenting(route: .one))
    }

    func testDismissGroup() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            case two
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horizontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)
        router.present(.two)
        router.dismiss(group: .horizontal)

        XCTAssertFalse(router.isPresenting(route: .one))
        XCTAssertFalse(router.isPresenting(route: .two))
        XCTAssertFalse(router.isPresenting(group: .horizontal))
    }
}

enum TestRouteGroup: AppRouteGroupProtocol {
    case horizontal
}
