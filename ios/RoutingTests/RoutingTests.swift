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
    private func createDelegate<T: AppRouteProtocol>(shouldPresent: Bool = true) -> RouterBlockDelegate<T> {
        let delegate = RouterBlockDelegate<T>()

        delegate.handleRoute = { _, _, completion in completion(Coordinator()) }
        delegate.shouldPresent = { _ in shouldPresent }

        return delegate
    }
}

// MARK: Horizontal flow tests

extension RoutingTests {
    func testPresentHorizontalRoute() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var isExclusive: Bool { false }
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horizontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)

        XCTAssertTrue(router.isPresenting(route: .one))
    }

    func testShouldDropHorizontalRoute() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var isExclusive: Bool { false }
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horizontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate(shouldPresent: false)
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)

        XCTAssertFalse(router.isPresenting(route: .one))
    }

    func testShouldDropIdenticalHorizontalRouteInSequence() throws {
        enum TestRoute: AppRouteProtocol {
            case one
            var isExclusive: Bool { false }
            var supportsSubNavigation: Bool { false }
            var routeGroup: TestRouteGroup { .horizontal }
        }

        let delegate: RouterBlockDelegate<TestRoute> = createDelegate()
        let router = ApplicationRouter<TestRoute>(delegate)

        router.present(.one)
        router.present(.one)

        XCTAssertEqual(router.presentedRoutes[.horizontal]?.count, 1)
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

    func testShouldDropSameLevelModalRouteIfPreceededByExclusive() throws {
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
    case horizontal, modalOne, modalTwo

    var isModal: Bool {
        switch self {
        case .horizontal:
            return false
        case .modalOne, .modalTwo:
            return true
        }
    }

    var modalLevel: Int {
        switch self {
        case .horizontal:
            return 0
        case .modalOne:
            return 1
        case .modalTwo:
            return 2
        }
    }
}
