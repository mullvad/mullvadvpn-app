//
//  LogStreamer.swift
//  MullvadVPN
//
//  Created by pronebird on 10/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

import Foundation

private let kLogPollIntervalSeconds = 2

/// A class that consolidates multiple log streams into one
class LogStreamer<Codec> where Codec: UnicodeCodec {
    private let fileURLs: [URL]
    private var remainingFileURLs: [URL]
    private var streams = [TextFileStream<Codec>]()
    private let queue = DispatchQueue(label: "net.mullvad.MullvadVPN.LogStreamer<\(Codec.self)>")
    private var retry: DispatchWorkItem?
    private var handlerBlock: ((String) -> Void)?

    init(fileURLs: [URL]) {
        self.fileURLs = fileURLs
        self.remainingFileURLs = fileURLs
    }

    func start(handler: @escaping (String) -> Void) {
        queue.async {
            self.handlerBlock = handler
            self.poll()
        }
    }

    func stop() {
        queue.async {
            self.retry?.cancel()
            self.handlerBlock = nil

            self.streams.removeAll()
            self.remainingFileURLs = self.fileURLs
        }
    }

    private func openRemainingStreams() -> Bool {
        var failedURLs = [URL]()
        for fileURL in remainingFileURLs {
            if let stream = TextFileStream<Codec>(fileURL: fileURL, separator: "\n") {
                streams.append(stream)

                didAddStream(stream)
            } else {
                failedURLs.append(fileURL)
            }
        }

        remainingFileURLs = failedURLs

        return failedURLs.isEmpty
    }

    private func poll() {
        if !self.openRemainingStreams() {
            self.scheduleRetry()
        }
    }

    private func scheduleRetry() {
        let workItem = DispatchWorkItem(block: { [weak self] in
            self?.poll()
        })
        queue.asyncAfter(wallDeadline: .now() + .seconds(kLogPollIntervalSeconds), execute: workItem)
        retry = workItem
    }

    private func didAddStream(_ stream: TextFileStream<Codec>) {
        stream.read { [weak self] (s) in
            guard let self = self else { return }

            self.queue.async {
                self.handlerBlock?(s)
            }
        }
    }
}
