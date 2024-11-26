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

    init(apiProxy: APIQuerying, tunnelManager: TunnelManager) {
        self.apiProxy = apiProxy
        self.tunnelManager = tunnelManager
        self.consolidatedLog = ConsolidatedApplicationLog(
            redactCustomStrings: [tunnelManager.deviceState.accountData?.number].compactMap { $0 },
            redactContainerPathsForSecurityGroupIdentifiers: [ApplicationConfiguration.securityGroupIdentifier]
        )
    }

    func loadLogFiles(completion: @escaping () -> Void) {
        DispatchQueue
            .global()
            .async { [weak self] in
                guard let self else { return }
                let logFileURLs = ApplicationTarget.allCases.flatMap {
                    ApplicationConfiguration.logFileURLs(for: $0, in: ApplicationConfiguration.containerURL)
                }
                consolidatedLog.addLogFiles(fileURLs: logFileURLs) {
                    DispatchQueue.main.async {
                        completion()
                    }
                }
            }
    }

    func fetchReportString(completion: @escaping (String) -> Void) {
        DispatchQueue
            .global()
            .async { [weak self] in
                guard let self else { return }
                let result = self.consolidatedLog.string
                DispatchQueue.main.async {
                    completion(result)
                }
            }
    }

    func sendReport(
        email: String,
        message: String,
        completion: @escaping (Result<Void, Error>) -> Void
    ) {
        DispatchQueue
            .global()
            .async { [weak self] in
                guard let self else { return }
                let logString = self.consolidatedLog.string
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
}
