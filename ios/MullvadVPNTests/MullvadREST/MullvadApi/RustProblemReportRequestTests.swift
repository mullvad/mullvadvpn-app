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

@testable import MullvadRustRuntime
@testable import MullvadTypes

struct RustProblemReportRequestTests {
    @Test(
        "Test vaild metadata insertion for SendProblemReport",
        arguments: [
            ["key1": "value1"],
            ["key2": "value2"],
            ["long_key_abcdefghijklmnopqrstuvwxyz": "long_value_1234567890"],
            ["special_chars_!@#$%": "special_value_(*&^%)"],
            ["": ""],
        ]
    )
    func testMetadataInsertion(metadata: [String: String]) {
        let request = ProblemReportRequest(
            address: "127.0.0.1",
            message: "Test message",
            log: "Log data",
            metadata: metadata
        )
        let rustRequest = RustProblemReportRequest(from: request)
        let rustStruct = rustRequest.toRust()
        #expect(rustStruct.metadata.inner != nil, "Metadata should not be nil for \(metadata)")
    }
}
