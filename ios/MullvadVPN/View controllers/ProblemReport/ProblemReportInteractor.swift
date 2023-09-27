//
//  ProblemReportInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 25/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Operations

final class ProblemReportInteractor {
    private let apiProxy: APIQuerying
    private let tunnelManager: TunnelManager

    private lazy var consolidatedLog: ConsolidatedApplicationLog = {
        let securityGroupIdentifier = ApplicationConfiguration.securityGroupIdentifier
        let redactStrings = [tunnelManager.deviceState.accountData?.number].compactMap { $0 }

        let report = ConsolidatedApplicationLog(
            redactCustomStrings: redactStrings,
            redactContainerPathsForSecurityGroupIdentifiers: [securityGroupIdentifier]
        )

        let logFileURLs = ApplicationTarget.allCases.map { ApplicationConfiguration.logFileURL(for: $0) }

        report.addLogFiles(fileURLs: logFileURLs, includeLogBackups: true)

        return report
    }()

    init(apiProxy: APIQuerying, tunnelManager: TunnelManager) {
        self.apiProxy = apiProxy
        self.tunnelManager = tunnelManager
    }

    func sendReport(
        email: String,
        message: String,
        completion: @escaping (Result<Void, Error>) -> Void
    ) -> Cancellable {
        let request = REST.ProblemReportRequest(
            address: email,
            message: message,
            log: consolidatedLog.string,
            metadata: consolidatedLog.metadata.reduce(into: [:]) { output, entry in
                output[entry.key.rawValue] = entry.value
            }
        )

        return apiProxy.sendProblemReport(
            request,
            retryStrategy: .default,
            completionHandler: completion
        )
    }

    var reportString: String {
        consolidatedLog.string
    }
}
