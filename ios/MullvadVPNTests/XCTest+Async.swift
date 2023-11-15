//
//  XCTest+Async.swift
//  MullvadVPNTests
//
//  Created by Mojgan on 2023-11-10.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import Foundation
import XCTest

extension XCTest {
    func XCTAssertThrowsErrorAsync<T: Sendable>(
        _ expression: @autoclosure () async throws -> T,
        _ message: @autoclosure () -> String = "",
        file: StaticString = #filePath,
        line: UInt = #line,
        _ errorHandler: (_ error: Error) -> Void = { _ in }
    ) async {
        do {
            _ = try await expression()
            XCTFail(message(), file: file, line: line)
        } catch {
            errorHandler(error)
        }
    }
}
