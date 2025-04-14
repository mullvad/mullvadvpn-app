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
        #expect(rustStruct.metadata != nil, "Metadata should not be for \(metadata)")
    }

    @Test("Test invalid metadata insertion for SendProblemReport")
    func testInvalidMetadataHandling() {
        let invalidMetadata: [[UInt8]: [UInt8]] = [
            [0xC0, 0x80]: [0xC0, 0x80], // // Incomplete UTF-8 byte sequence for key an value
            [0x7E]: [0x80], // Valid key , but invalid start byte in UTF-8
            [0xE0, 0x80]: [0xC2, 0x80], // Malformed UTF-8 multibyte sequence for key and valid value
        ]
        let metadata = swift_problem_report_metadata_new()
        for (keyBytes, valueBytes) in invalidMetadata {
            keyBytes.withUnsafeBytes { (keyPtr: UnsafeRawBufferPointer) in
                valueBytes.withUnsafeBytes { (valuePtr: UnsafeRawBufferPointer) in
                    guard let keyBaseAddress = keyPtr.baseAddress?.assumingMemoryBound(to: UInt8.self),
                          let valueBaseAddress = valuePtr.baseAddress?.assumingMemoryBound(to: UInt8.self) else {
                        return
                    }

                    let result = swift_problem_report_metadata_add(
                        metadata,
                        keyBaseAddress,
                        valueBaseAddress
                    )

                    #expect(
                        result == false,
                        "Metadata with invalid UTF-8 should not be added. Key/Value: [\(keyBytes): \(valueBytes)]"
                    )
                }
            }
        }
    }
}
