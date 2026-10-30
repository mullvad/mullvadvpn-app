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

struct SpeedTestResult: Sendable {
    let downloadMbps: Double
}

struct SpeedTestProgress: Sendable {
    let downloadMbps: Double
    let progress: Double
}

enum SpeedTestError: Error {
    case invalidResponse
    case invalidDuration
    case invalidURL
}

final class SpeedTest: NSObject, Sendable {

    private let connectionCount = 4
    private let warmupDuration: TimeInterval = 1
    private let measurementDuration: TimeInterval = 8

    private let state: SpeedTestState

    private let session: URLSession

    override init() {
        let configuration = URLSessionConfiguration.ephemeral
        configuration.requestCachePolicy = .reloadIgnoringLocalCacheData
        configuration.urlCache = nil
        configuration.timeoutIntervalForRequest = 30
        configuration.timeoutIntervalForResource = 30

        let state = SpeedTestState()

        self.state = state

        self.session = URLSession(
            configuration: configuration,
            delegate: state,
            delegateQueue: nil
        )

        super.init()
    }

    deinit {
        session.invalidateAndCancel()
    }

    func testDownload(
        progress: (@Sendable (SpeedTestProgress) -> Void)? = nil
    ) async throws -> SpeedTestResult {

        try await state.runTest(
            session: session,
            connectionCount: connectionCount,
            warmupDuration: warmupDuration,
            measurementDuration: measurementDuration,
            progress: progress
        )
    }
}

private actor SpeedTestState: NSObject, URLSessionDataDelegate {

    private var tasks: [URLSessionDataTask] = []

    private var measurementBytes: Int64 = 0
    private var testStart: Date?
    private var measurementStart: Date?

    private var isMeasuring = false
    private var isFinished = false

    private var continuation: CheckedContinuation<SpeedTestResult, Error>?
    private var progressHandler: (@Sendable (SpeedTestProgress) -> Void)?

    func runTest(
        session: URLSession,
        connectionCount: Int,
        warmupDuration: TimeInterval,
        measurementDuration: TimeInterval,
        progress: (@Sendable (SpeedTestProgress) -> Void)?
    ) async throws -> SpeedTestResult {

        stop()

        measurementBytes = 0
        isMeasuring = false
        isFinished = false
        testStart = Date()
        measurementStart = nil
        progressHandler = progress

        return try await withTaskCancellationHandler {
            try await withCheckedThrowingContinuation { continuation in

                self.continuation = continuation

                guard
                     let url = URL(string: "https://speed.cloudflare.com/__down?bytes=250000000")
                else {
                    finish(.failure(SpeedTestError.invalidURL))
                    return
                }

                for _ in 0..<connectionCount {
                    var request = URLRequest(url: url)
                    request.httpMethod = "GET"
                    request.cachePolicy = .reloadIgnoringLocalCacheData

                    let task = session.dataTask(with: request)
                    tasks.append(task)
                    task.resume()
                }
            }
        } onCancel: {
            Task {
                await self.cancel()
            }
        }
    }

    private func cancel() {
        finish(.failure(CancellationError()))
    }

    private func startMeasurement(
        duration: TimeInterval
    ) {
        guard !isMeasuring, !isFinished else {
            return
        }

        isMeasuring = true
        measurementBytes = 0
        measurementStart = Date()

        emitProgress()

        Task {
            try? await Task.sleep(
                for: .seconds(duration)
            )

            finishMeasurement()
        }
    }

    private func finishMeasurement() {
        guard
            isMeasuring,
            !isFinished,
            let measurementStart
        else {
            return
        }

        isMeasuring = false

        let elapsed = Date().timeIntervalSince(measurementStart)

        guard elapsed > 0 else {
            finish(.failure(SpeedTestError.invalidDuration))
            return
        }

        let bits = Double(measurementBytes) * 8
        let mbps = bits / elapsed / 1_000_000

        finish(
            .success(
                SpeedTestResult(downloadMbps: mbps)
            )
        )
    }

    private func emitProgress() {
        guard isMeasuring,
              let measurementStart else {
            return
        }

        let elapsed = Date().timeIntervalSince(measurementStart)
        guard elapsed > 0 else { return }

        let mbps = Double(measurementBytes) * 8
            / elapsed / 1_000_000

        print("Bytes: \(measurementBytes), Elapsed: \(elapsed), Speed: \(mbps) Mbps")

        let progress = min(elapsed / 8, 1)

        progressHandler?(
            SpeedTestProgress(
                downloadMbps: mbps,
                progress: progress
            )
        )
    }

    private func stop() {
        tasks.forEach { $0.cancel() }
        tasks.removeAll()
    }

    private func finish(
        _ result: Result<SpeedTestResult, Error>
    ) {
        guard !isFinished else {
            return
        }

        isFinished = true
        stop()

        let continuation = self.continuation
        self.continuation = nil

        continuation?.resume(with: result)
    }

    nonisolated func urlSession(
        _ session: URLSession,
        dataTask: URLSessionDataTask,
        didReceive data: Data
    ) {
        let byteCount = Int64(data.count)
        print("Received chunk: \(byteCount) bytes")

        Task {
            await receiveData(byteCount)
        }
    }
    
    nonisolated func urlSession(
        _ session: URLSession,
        dataTask: URLSessionDataTask,
        didReceive response: URLResponse,
        completionHandler: @escaping (URLSession.ResponseDisposition) -> Void
    ) {
        if let httpResponse = response as? HTTPURLResponse {
            print("HTTP status:", httpResponse.statusCode)
            print("Content-Length:", httpResponse.expectedContentLength)
            print("MIME type:", httpResponse.mimeType ?? "unknown")
        }

        completionHandler(.allow)
    }

    private func receiveData(_ byteCount: Int64) {
        guard !isFinished else {
            return
        }

        let now = Date()

        if !isMeasuring,
            let testStart,
            now.timeIntervalSince(testStart) >= 2
        {
            startMeasurement(duration: 8)
        }

        if isMeasuring {
            measurementBytes += byteCount
            emitProgress()
        }
    }

    nonisolated func urlSession(
        _ session: URLSession,
        task: URLSessionTask,
        didCompleteWithError error: Error?
    ) {
        guard let error else {
            return
        }

        if (error as NSError).code == NSURLErrorCancelled {
            return
        }

        Task {
            await handleError(error)
        }
    }

    private func handleError(_ error: Error) {
        finish(.failure(error))
    }
}
