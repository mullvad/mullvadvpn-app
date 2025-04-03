//
//  RustProblemReportRequestTests.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-26.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

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
        #expect(rustStruct.meta_data != nil, "Metadata should not be nil for \(metadata)")
    }
}
