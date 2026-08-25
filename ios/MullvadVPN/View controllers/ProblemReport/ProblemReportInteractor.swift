// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations

final class ProblemReportInteractor: @unchecked Sendable {
    private let apiProxy: APIQuerying
    private let tunnelManager: TunnelManager
    private let consolidatedLog: ConsolidatedApplicationLog
    private var reportedString = ""

    init(apiProxy: APIQuerying, tunnelManager: TunnelManager, redactor: LogRedacting?) {
        self.apiProxy = apiProxy
        self.tunnelManager = tunnelManager
        self.consolidatedLog = ConsolidatedApplicationLog(
            redactor: redactor,
            bufferSize: ApplicationConfiguration.logMaximumFileSize
        )
    }

    func fetchReportString() async -> String {
        await withCheckedContinuation { continuation in
            consolidatedLog.addLogFiles(
                fileURLs: ApplicationTarget.allCases.flatMap {
                    ApplicationConfiguration.logFileURLs(
                        for: $0,
                        in: ApplicationConfiguration.containerURL
                    )
                }
            ) { [weak self] in
                guard let self else {
                    continuation.resume(returning: "")
                    return
                }

                continuation.resume(
                    returning: consolidatedLog.string
                )
            }
        }
    }

    func sendReport(
        email: String,
        message: String,
        includeAccountTokenInLogs: Bool
    ) async -> Result<Void, Error> {
        let logString = consolidatedLog.string

        let accountToken =
            if isUserLoggedIn(),
                includeAccountTokenInLogs,
                let token = tunnelManager.deviceState.accountData?.identifier
            {
                "\naccount-token: \(token)"
            } else {
                ""
            }

        let updatedLogString: String

        if logString.isEmpty {
            updatedLogString = await fetchReportString()
        } else {
            updatedLogString = logString
        }

        return await sendProblemReport(
            email: email,
            message: message + accountToken,
            logString: updatedLogString
        )
    }

    func isUserLoggedIn() -> Bool {
        tunnelManager.deviceState.isLoggedIn
    }

    private func sendProblemReport(
        email: String,
        message: String,
        logString: String
    ) async -> Result<Void, Error> {
        let metadataDict = consolidatedLog.metadata.reduce(into: [:]) { output, entry in
            output[entry.key.rawValue] = entry.value
        }

        let request = ProblemReportRequest(
            address: email,
            message: message,
            log: logString,
            metadata: metadataDict
        )

        return await apiProxy.sendProblemReport(
            request,
            retryStrategy: .default
        )
    }
}
