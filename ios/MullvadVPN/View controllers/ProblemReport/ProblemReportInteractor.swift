//
//  ProblemReportInteractor.swift
//  MullvadVPN
//
//  Created by pronebird on 25/10/2022.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadREST
import MullvadTypes
import Operations

final class ProblemReportInteractor: @unchecked Sendable {
    private let apiProxy: APIQuerying
    private let tunnelManager: TunnelManager
    private let consolidatedLog: ConsolidatedApplicationLog
    private var reportedString = ""
    private var requestCancellable: Cancellable?

    init(apiProxy: APIQuerying, tunnelManager: TunnelManager) {
        self.apiProxy = apiProxy
        self.tunnelManager = tunnelManager
        let redactCustomStrings = [tunnelManager.deviceState.accountData?.number].compactMap { $0 }
        self.consolidatedLog = ConsolidatedApplicationLog(
            redactCustomStrings: redactCustomStrings.isEmpty ? nil : redactCustomStrings,
            redactContainerPathsForSecurityGroupIdentifiers: [ApplicationConfiguration.securityGroupIdentifier],
            bufferSize: ApplicationConfiguration.logMaximumFileSize
        )
    }

    func fetchReportString(completion: @escaping @Sendable (Result<String, Error>) -> Void) {
        consolidatedLog.addLogFiles(fileURLs: ApplicationTarget.allCases.flatMap {
            ApplicationConfiguration.logFileURLs(for: $0, in: ApplicationConfiguration.containerURL)
        }, completion: completion)
    }

    func sendReport(
        email: String,
        message: String,
        completion: @escaping @Sendable (Result<Void, Error>) -> Void
    ) {
        let logString = self.consolidatedLog.string
        if logString.isEmpty {
            fetchReportString { [weak self] result in
                switch result {
                case let .success(logString):
                    self?.sendProblemReport(
                        email: email,
                        message: message,
                        logString: logString,
                        completion: completion
                    )
                case let .failure(error):
                    completion(.failure(error))
                }
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

    func cancelSendingReport() {
        consolidatedLog.cancel()
        requestCancellable?.cancel()
    }

    private func sendProblemReport(
        email: String,
        message: String,
        logString: String,
        completion: @escaping @Sendable (Result<Void, Error>) -> Void
    ) {
        let metadataDict = self.consolidatedLog.metadata.reduce(into: [:]) { output, entry in
            output[entry.key.rawValue] = entry.value
        }

        let request = ProblemReportRequest(
            address: email,
            message: message,
            log: logString,
            metadata: metadataDict
        )

        requestCancellable = self.apiProxy.sendProblemReport(request, retryStrategy: .default) { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
}
