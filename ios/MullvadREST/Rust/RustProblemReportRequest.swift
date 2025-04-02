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
    private let logger = Logger(label: "RustProblemReportRequest")
    private let problemReportMetaData: ProblemReportMetadata
    private let addressPointer: UnsafePointer<UInt8>?
    private let messagePointer: UnsafePointer<UInt8>?
    private let logPointer: UnsafePointer<UInt8>?
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
            address: addressPointer,
            message: messagePointer,
            log: logPointer,
            meta_data: problemReportMetaData
        )
    }

    deinit {
        swift_problem_report_meta_data_free(problemReportMetaData)
    }
}
