//
//  RustProblemReportRequest.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadLogging
import MullvadTypes

final public class RustProblemReportRequest {
    private let logger = Logger(label: "RustProblemReportRequest")
    private let addressPointer: UnsafePointer<CChar>?
    private let messagePointer: UnsafePointer<CChar>?
    private let logPointer: UnsafePointer<CChar>?
    private let problemReportMetaData: ProblemReportMetadata

    public init(from request: ProblemReportRequest) {
        self.problemReportMetaData = swift_problem_report_meta_data_new()
        self.addressPointer = request.address.toCStringPointer()
        self.messagePointer = request.message.toCStringPointer()
        self.logPointer = request.log.toCStringPointer()

        for (key, value) in request.metadata {
            let isAdded = swift_problem_report_meta_data_add(problemReportMetaData, key, value)
            if !isAdded {
                logger
                    .error("Failed to add metadata. Key: '\(key)' might be invalid or contain unsupported characters.")
            }
        }
    }

    public func toRust() -> SwiftProblemReportRequest {
        SwiftProblemReportRequest(
            address: addressPointer,
            message: messagePointer,
            log: logPointer,
            meta_data: problemReportMetaData
        )
    }

    deinit {
        swift_problem_report_meta_data_free(problemReportMetaData)
        addressPointer?.deallocate()
        messagePointer?.deallocate()
        logPointer?.deallocate()
    }
}
