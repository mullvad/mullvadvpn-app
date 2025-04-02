//
//  RustProblemReportRequest.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-03-21.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import MullvadLogging
import MullvadREST

final public class RustProblemReportRequest {
    private let logger = Logger(label: "RustProblemReportRequest")
    private let request: REST.ProblemReportRequest
    private let problemReportMetaData: ProblemReportMetadata

    public init(from request: REST.ProblemReportRequest) {
        self.request = request
        self.problemReportMetaData = swift_problem_report_meta_data_new()

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
            address: request.address.withCStringPointer(),
            message: request.message.withCStringPointer(),
            log: request.log.withCStringPointer(),
            meta_data: problemReportMetaData
        )
    }

    deinit {
        swift_problem_report_meta_data_free(problemReportMetaData)
    }
}
