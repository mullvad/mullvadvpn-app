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
import MullvadRustRuntime
import MullvadTypes
import Operations

protocol ProblemReportInteractorProtocol: Sendable {
    func fetchReportString(completion: @escaping @Sendable (String) -> Void)

    func sendReport(
        email: String,
        message: String,
        includeAccountTokenInLogs: Bool,
        completion: @escaping @Sendable (Result<Void, Error>) -> Void
    )
}

extension ProblemReportInteractorProtocol {
    func fetchReportString() async -> String {
        await withCheckedContinuation { continuation in
            self.fetchReportString { continuation.resume(returning: $0) }
        }
    }

    func sendReport(email: String, message: String, includeAccountTokenInLogs: Bool) async throws {
        try await withCheckedThrowingContinuation { continuation in
            self.sendReport(
                email: email,
                message: message,
                includeAccountTokenInLogs: includeAccountTokenInLogs
            ) { continuation.resume(with: $0) }
        }
    }
}

final class ProblemReportInteractor: ProblemReportInteractorProtocol, @unchecked Sendable {
    private let apiProxy: APIQuerying
    private let tunnelManager: TunnelManager
    private let consolidatedLog: ConsolidatedApplicationLog
    private var reportedString = ""
    private var requestCancellable: Cancellable?

    init(apiProxy: APIQuerying, tunnelManager: TunnelManager, redactor: LogRedacting?) {
        self.apiProxy = apiProxy
        self.tunnelManager = tunnelManager
        self.consolidatedLog = ConsolidatedApplicationLog(
            redactor: redactor,
            bufferSize: ApplicationConfiguration.logMaximumFileSize
        )
    }

    func fetchReportString(completion: @escaping @Sendable (String) -> Void) {
        let existing = consolidatedLog.string
        guard existing.isEmpty else {
            completion(existing)
            return
        }

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
        let accountToken =
            if isUserLoggedIn() && includeAccountTokenInLogs,
                let token = tunnelManager.deviceState.accountData?.identifier
            {
                "\naccount-token: \(token)"
            } else { "" }

        fetchReportString { [weak self] logString in
            self?.sendProblemReport(
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

// MARK: A mock interactor for SwiftUI Previews

struct MockProblemReportInteractor: ProblemReportInteractorProtocol {
    var reportError: (any Error)?

    func fetchReportString(completion: @escaping @Sendable (String) -> Void) {
        completion(
            """
            The log file will go here
            =========================

            Something something something...
            """
        )
    }

    func sendReport(
        email: String, message: String, includeAccountTokenInLogs: Bool,
        completion: @escaping (Result<Void, any Error>) -> Void
    ) {
        //  try await Task.sleep(nanoseconds: 1_000_000_000)
        if let reportError {
            completion(.failure(reportError))
        } else {
            completion(.success(()))
        }
    }
}
