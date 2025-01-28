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
}

enum TestRouteGroup: AppRouteGroupProtocol {
    case horizontal
}
