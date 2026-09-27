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

final actor SpeedTest: NSObject {

    private let connectionCount = 4
    private let warmupDuration: TimeInterval = 2
    private let measurementDuration: TimeInterval = 8

    private var session: URLSession!
    private var tasks: [URLSessionDataTask] = []

    private var totalBytes: Int64 = 0
    private var measurementBytes: Int64 = 0

    private var testStart: Date?
    private var measurementStart: Date?

    private var isMeasuring = false

    private var continuation: CheckedContinuation<SpeedTestResult, Error>?
    private var progressHandler: (@Sendable (SpeedTestProgress) -> Void)?

    override init() {
        super.init()

        let configuration = URLSessionConfiguration.ephemeral
        configuration.requestCachePolicy = .reloadIgnoringLocalCacheData
        configuration.urlCache = nil
        configuration.timeoutIntervalForRequest = 30
        configuration.timeoutIntervalForResource = 30

        session = URLSession(
            configuration: configuration,
            delegate: self,
            delegateQueue: nil
        )
    }

    deinit {
        session.invalidateAndCancel()
    }

    func testDownload(
        progress: (@Sendable (SpeedTestProgress) -> Void)? = nil
    ) async throws -> SpeedTestResult {

        stop()

        progressHandler = progress

        totalBytes = 0
        measurementBytes = 0
        isMeasuring = false

        testStart = Date()

        return try await withTaskCancellationHandler {

            try await withCheckedThrowingContinuation {
                (continuation: CheckedContinuation<SpeedTestResult, Error>) in

                self.continuation = continuation
                self.startConnections()
            }

        } onCancel: {
            self.stop()

            self.continuation?.resume(
                throwing: CancellationError()
            )

            self.continuation = nil
        }
    }

    private func startConnections() {

        guard let url = URL(
            string: "https://speed.cloudflare.com/__down?bytes=250000000"
        ) else {
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

    private func startMeasurement() {
        guard !isMeasuring else {
            return
        }

        isMeasuring = true
        measurementBytes = 0
        measurementStart = Date()

        emitProgress()

        DispatchQueue.global().asyncAfter(
            deadline: .now() + measurementDuration
        ) { [weak self] in

            self?.finishMeasurement()
        }
    }

    private func finishMeasurement() {
        guard isMeasuring, let measurementStart  else {
            return
        }

        isMeasuring = false

        let elapsed = Date().timeIntervalSince(measurementStart)
        let bytes = measurementBytes

        guard elapsed > 0 else {
            finish(.failure(SpeedTestError.invalidDuration))
            return
        }

        let bits = Double(bytes) * 8
        let mbps = bits / elapsed / 1_000_000

        finish(
            .success(
                SpeedTestResult(
                    downloadMbps: mbps
                )
            )
        )
    }

    private func emitProgress() {
        let bytes = measurementBytes
        let start = measurementStart

        guard
            let start,
            isMeasuring
        else {
            return
        }

        let elapsed = Date().timeIntervalSince(start)

        guard elapsed > 0 else {
            return
        }

        let mbps =
            Double(bytes) * 8
            / elapsed
            / 1_000_000

        let progress = min(
            elapsed / measurementDuration,
            1
        )

        progressHandler?(
            SpeedTestProgress(
                downloadMbps: mbps,
                progress: progress
            )
        )
    }

    private func stop() {
        tasks.forEach {
            $0.cancel()
        }

        tasks.removeAll()
    }

    private func finish(
        _ result: Result<SpeedTestResult, Error>
    ) {

        stop()

        let continuation = self.continuation
        self.continuation = nil

        continuation?.resume(with: result)
    }
}


//extension SpeedTest: URLSessionDataDelegate {
//
//    func urlSession(
//        _ session: URLSession,
//        dataTask: URLSessionDataTask,
//        didReceive data: Data
//    ) {
//
//        let byteCount = Int64(data.count)
//
//        lock.lock()
//
//        totalBytes += byteCount
//
//        if isMeasuring {
//            measurementBytes += byteCount
//        }
//
//        let shouldStartMeasurement =
//            !isMeasuring &&
//            Date().timeIntervalSince(testStart ?? Date())
//            >= warmupDuration
//
//        lock.unlock()
//
//        if shouldStartMeasurement {
//            startMeasurement()
//        }
//
//        if isMeasuring {
//            emitProgress()
//        }
//    }
//
//    func urlSession(
//        _ session: URLSession,
//        task: URLSessionTask,
//        didCompleteWithError error: Error?
//    ) {
//
//        if let error {
//
//            if (error as NSError).code ==
//                NSURLErrorCancelled {
//                return
//            }
//
//            finish(.failure(error))
//        }
//    }
//}
