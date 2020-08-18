//
//  LogStreamer.swift
//  MullvadVPN
//
//  Created by pronebird on 10/08/2020.
//  Copyright © 2020 Mullvad VPN AB. All rights reserved.
//

#if DEBUG

import Foundation

private let kLogPollIntervalSeconds = 2

/// A class that consolidates multiple log streams into one
class LogStreamer<Codec> where Codec: UnicodeCodec {
    private let fileURLs: [URL]
    private var remainingFileURLs: [URL]
    private var streams = [TextFileStream<Codec>]()
    private var eventSources = [DispatchSourceFileSystemObject]()
    private let queue = DispatchQueue(label: "net.mullvad.MullvadVPN.LogStreamer<\(Codec.self)>")
    private var retry: DispatchWorkItem?
    private var handlerBlock: ((String) -> Void)?
    private var isStarted = false

    init(fileURLs: [URL]) {
        self.fileURLs = fileURLs
        self.remainingFileURLs = fileURLs
    }

    func start(handler: @escaping (String) -> Void) {
        queue.async {
            guard !self.isStarted else { return }

            self.isStarted = true
            self.handlerBlock = handler
            self.poll()
        }
    }

    func stop() {
        queue.async {
            guard self.isStarted else { return }

            self.isStarted = false

            self.retry?.cancel()
            self.handlerBlock = nil

            self.eventSources.removeAll()
            self.streams.removeAll()
            self.remainingFileURLs = self.fileURLs
        }
    }

    private func openRemainingStreams() -> Bool {
        var failedURLs = [URL]()
        for fileURL in remainingFileURLs {
            if let stream = TextFileStream<Codec>(fileURL: fileURL, separator: "\n") {
                streams.append(stream)

                stream.read { [weak self] (s) in
                    guard let self = self else { return }

                    self.queue.async {
                        self.handlerBlock?(s)
                    }
                }

                addFileWatch(fileURL: fileURL, stream: stream)
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
            guard let self = self, self.isStarted else { return }

            self.poll()
        })
        queue.asyncAfter(wallDeadline: .now() + .seconds(kLogPollIntervalSeconds), execute: workItem)
        retry = workItem
    }

    /// Watch file renames and re-add the stream once that happens
    private func addFileWatch(fileURL: URL, stream: TextFileStream<Codec>) {
        let source = DispatchSource.makeFileSystemObjectSource(
            fileDescriptor: stream.fileDescriptor,
            eventMask: .rename,
            queue: queue
        )

        source.setEventHandler { [weak self, weak source] in
            guard let self = self, self.isStarted else { return }

            // Cancel current event source
            source?.cancel()

            // Release the stream
            self.streams.removeAll { (s) -> Bool in
                return stream === s
            }

            // Add the file URL to backlog & poll
            self.remainingFileURLs.append(fileURL)
            self.poll()
        }

        source.activate()

        eventSources.append(source)
    }
}

#endif
