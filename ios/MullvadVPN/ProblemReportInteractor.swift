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
    private let apiProxy: REST.APIProxy
    private let tunnelManager: TunnelManager

    private lazy var consolidatedLog: ConsolidatedApplicationLog = {
        let securityGroupIdentifier = ApplicationConfiguration.securityGroupIdentifier

        // TODO: make sure we redact old tokens

        let redactStrings = [tunnelManager.deviceState.accountData?.number].compactMap { $0 }

        let report = ConsolidatedApplicationLog(
            redactCustomStrings: redactStrings,
            redactContainerPathsForSecurityGroupIdentifiers: [securityGroupIdentifier]
        )

        report.addLogFiles(fileURLs: ApplicationConfiguration.logFileURLs, includeLogBackups: true)

        return report
    }()

    init(apiProxy: REST.APIProxy, tunnelManager: TunnelManager) {
        self.apiProxy = apiProxy
        self.tunnelManager = tunnelManager
    }

    func sendReport(
        email: String,
        message: String,
        completion: @escaping (OperationCompletion<Void, REST.Error>) -> Void
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
        return consolidatedLog.string
    }
}
