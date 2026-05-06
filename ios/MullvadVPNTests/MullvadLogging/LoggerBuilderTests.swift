//
//  LoggedBuilderTests.swift
//  MullvadVPNTests
//
//  Created by Marco Nikic on 2025-11-04.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Testing

@testable import MullvadLogging
@testable import MullvadRustRuntime

struct LoggerBuilderTests {

    @Test func installIsIdempotent() async throws {
        let redactedLogger = RustLogRedactor()

        LoggerBuilder.shared.install(redactedLogger)
        // This should crash if the `install` function is not idempotent
        LoggerBuilder.shared.install(redactedLogger)
        LoggerBuilder.shared.install(redactedLogger)
    }
}
