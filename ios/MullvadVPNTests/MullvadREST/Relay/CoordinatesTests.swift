//
//  CoordinatesTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2023-07-11.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import CoreLocation
@testable import MullvadREST
import XCTest

final class CoordinatesTests: XCTestCase {
    func testHaversine() {
        let distance1 = Haversine.distance(36.12, -86.67, 33.94, -118.4)
        XCTAssertEqual(2887.259_950_607_108_7, distance1)

        let distance2 = Haversine.distance(90.0, 5.0, 90.0, 79.0)
        XCTAssertEqual(0.0000000000004696822692507987, distance2)

        let distance3 = Haversine.distance(0, 0, 0, 0)
        XCTAssertEqual(0, distance3)

        let distance4 = Haversine.distance(49.0, 12.0, 49.0, 12.0)
        XCTAssertEqual(0, distance4)

        let distance5 = Haversine.distance(6.0, 27.0, 7.0, 27.0)
        XCTAssertEqual(111.226_342_571_094_7, distance5)

        let distance6 = Haversine.distance(0.0, 179.5, 0.0, -179.5)
        XCTAssertEqual(111.226_342_571_100_6, distance6)
    }

    func testMidpoint() {
        let midpoint1 = Midpoint.location(
            in: [
                CLLocationCoordinate2D(latitude: 0, longitude: 90),
                CLLocationCoordinate2D(latitude: 90, longitude: 0),
            ]
        )

        let midpoint2 = Midpoint.location(
            in: [
                CLLocationCoordinate2D(latitude: -20, longitude: 90),
                CLLocationCoordinate2D(latitude: -20, longitude: -90),
            ]
        )

        let expectedMidpoint1Value = CLLocationCoordinate2D(latitude: 45, longitude: 90)
        XCTAssertEqual(expectedMidpoint1Value.latitude, midpoint1.latitude, accuracy: 0.1)
        XCTAssertEqual(expectedMidpoint1Value.longitude, midpoint1.longitude, accuracy: 0.1)

        let expectedMidpoint2Value = CLLocationCoordinate2D(latitude: -90, longitude: 0)
        XCTAssertEqual(expectedMidpoint2Value.latitude, midpoint2.latitude, accuracy: 0.1)
        XCTAssertEqual(expectedMidpoint2Value.longitude, midpoint2.longitude, accuracy: 0.1)
    }
}

extension CLLocationCoordinate2D: Equatable {
    public static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.latitude == rhs.latitude && lhs.longitude == rhs.longitude
    }
}
