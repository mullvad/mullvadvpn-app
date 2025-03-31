//
//  RustProblemReportRequest.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadLogging
import MullvadRustRuntime

final public class RustProblemReportRequest {
    typealias StringPointer = (pointer: UnsafePointer<UInt8>?, size: UInt)?
    private let logger = Logger(label: "RustProblemReportRequest")
    private let problemReportMetaData: ProblemReportMetadata
    private let addressPointer: StringPointer
    private let messagePointer: StringPointer
    private let logPointer: StringPointer
    public init(from request: REST.ProblemReportRequest) {
        addressPointer = request.address.withCStringPointer()
        messagePointer = request.message.withCStringPointer()
        logPointer = request.log.withCStringPointer()

        self.problemReportMetaData = swift_problem_report_meta_data_new()

        for (key, value) in request.metadata {
            key.withCString { keyPtr in
                value.withCString { valuePtr in
                    if swift_problem_report_meta_data_add(problemReportMetaData, keyPtr, valuePtr) == false {
                        logger
                            .error(
                                "Failed to add metadata. Key: '\(key)' might be invalid or contain unsupported characters."
                            )
                    }
                }
            }
        }
    }

    public func toRust() -> SwiftProblemReportRequest {
        return SwiftProblemReportRequest(
            address: addressPointer?.pointer ?? nil,
            address_len: addressPointer?.size ?? 0,
            message: messagePointer?.pointer ?? nil,
            message_len: messagePointer?.size ?? 0,
            log: logPointer?.pointer ?? nil,
            log_len: logPointer?.size ?? 0,
            meta_data: problemReportMetaData
        )
    }

    deinit {
        swift_problem_report_meta_data_free(problemReportMetaData)
    }
}
