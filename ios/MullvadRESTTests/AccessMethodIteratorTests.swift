//
//  AccessMethodIteratorTests.swift
//  MullvadRESTTests
//
//  Created by Mojgan on 2024-01-10.
//  Copyright © 2024 Mullvad VPN AB. All rights reserved.
//

@testable import MullvadREST
@testable import MullvadSettings
@testable import MullvadTypes
import XCTest

final class AccessMethodIteratorTests: XCTestCase {
    var userDefaults: UserDefaults!
    static var suiteName: String!

    override class func setUp() {
        super.setUp()
        suiteName = UUID().uuidString
    }

    override func tearDownWithError() throws {
        userDefaults.removePersistentDomain(forName: Self.suiteName)
        try super.tearDownWithError()
    }
}
