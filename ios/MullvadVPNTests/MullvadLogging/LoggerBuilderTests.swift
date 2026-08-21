// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

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
