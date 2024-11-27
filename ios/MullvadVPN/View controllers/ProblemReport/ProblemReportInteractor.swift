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
    private let consolidatedLog: ConsolidatedApplicationLog
    private var reportedString = ""

    init(apiProxy: APIQuerying, tunnelManager: TunnelManager) {
        self.apiProxy = apiProxy
        self.tunnelManager = tunnelManager
        self.consolidatedLog = ConsolidatedApplicationLog(
            redactCustomStrings: [tunnelManager.deviceState.accountData?.number].compactMap { $0 },
            redactContainerPathsForSecurityGroupIdentifiers: [ApplicationConfiguration.securityGroupIdentifier],
            bufferSize: ApplicationConfiguration.logMaximumFileSize
        )
    }

    func fetchReportString(completion: @escaping (String) -> Void) {
        consolidatedLog.addLogFiles(fileURLs: ApplicationTarget.allCases.flatMap {
            ApplicationConfiguration.logFileURLs(for: $0, in: ApplicationConfiguration.containerURL)
        }) { [weak self] in
            guard let self else { return }
            completion(consolidatedLog.string)
        }
    }

    func sendReport(
        email: String,
        message: String,
        completion: @escaping (Result<Void, Error>) -> Void
    ) {
        let logString = self.consolidatedLog.string

        if logString.isEmpty {
            fetchReportString { [weak self] updatedLogString in
                self?.sendProblemReport(
                    email: email,
                    message: message,
                    logString: updatedLogString,
                    completion: completion
                )
            }
        } else {
            sendProblemReport(
                email: email,
                message: message,
                logString: logString,
                completion: completion
            )
        }
    }

    private func sendProblemReport(
        email: String,
        message: String,
        logString: String,
        completion: @escaping (Result<Void, Error>) -> Void
    ) {
        let metadataDict = self.consolidatedLog.metadata.reduce(into: [:]) { output, entry in
            output[entry.key.rawValue] = entry.value
        }

        let request = REST.ProblemReportRequest(
            address: email,
            message: message,
            log: logString,
            metadata: metadataDict
        )

        _ = self.apiProxy.sendProblemReport(
            request,
            retryStrategy: .default
        ) { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
}
