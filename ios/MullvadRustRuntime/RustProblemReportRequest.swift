// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only
import MullvadLogging
import MullvadTypes

final public class RustProblemReportRequest {
    private let logger = Logger(label: "RustProblemReportRequest")
    private let addressPointer: UnsafePointer<CChar>?
    private let messagePointer: UnsafePointer<CChar>?
    private let logPointer: UnsafePointer<CChar>?
    private let problemReportMetaData: ProblemReportMetadata

    public init(from request: ProblemReportRequest) {
        self.problemReportMetaData = swift_problem_report_metadata_new()
        self.addressPointer = request.address.toCStringPointer()
        self.messagePointer = request.message.toCStringPointer()
        self.logPointer = request.log.toCStringPointer()

        for (key, value) in request.metadata {
            let isAdded = swift_problem_report_metadata_add(problemReportMetaData, key, value)
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
            metadata: problemReportMetaData
        )
    }

    deinit {
        swift_problem_report_metadata_free(problemReportMetaData)
        addressPointer?.deallocate()
        messagePointer?.deallocate()
        logPointer?.deallocate()
    }
}
