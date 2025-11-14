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

    func fetchReportString(completion: @escaping @Sendable (String) -> Void) {
        consolidatedLog.addLogFiles(
            fileURLs: ApplicationTarget.allCases.flatMap {
                ApplicationConfiguration.logFileURLs(for: $0, in: ApplicationConfiguration.containerURL)
            }
        ) { [weak self] in
            guard let self else { return }
            completion(consolidatedLog.string)
        }
    }

    func sendReport(
        email: String,
        message: String,
        includeAccountTokenInLogs: Bool,
        completion: @escaping @Sendable (Result<Void, Error>) -> Void
    ) {
        let logString = self.consolidatedLog.string
        let accountToken =
            if isUserLoggedIn() && includeAccountTokenInLogs,
                let token = tunnelManager.deviceState.accountData?.identifier
            {
                "\naccount-token: \(token)"
            } else { "" }

        if logString.isEmpty {
            fetchReportString { [weak self, accountToken] updatedLogString in
                self?.sendProblemReport(
                    email: email,
                    message: message + accountToken,
                    logString: updatedLogString,
                    completion: completion
                )
            }
        } else {
            sendProblemReport(
                email: email,
                message: message + accountToken,
                logString: logString,
                completion: completion
            )
        }
    }

    func isUserLoggedIn() -> Bool {
        tunnelManager.deviceState.isLoggedIn
    }

    func cancelSendingReport() {
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

        requestCancellable = apiProxy.sendProblemReport(request, retryStrategy: .default) { result in
            DispatchQueue.main.async {
                completion(result)
            }
        }
    }
}
