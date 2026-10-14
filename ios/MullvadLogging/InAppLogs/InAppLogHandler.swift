// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import MullvadTypes

public final class InAppLogBlockObserver: @unchecked Sendable {
    public typealias DidAddLogEntryHandler = (InAppLogEntry) -> Void

    public var didAddLogEntryHandler: DidAddLogEntryHandler?

    public init(didAddLogEntryHandler: DidAddLogEntryHandler? = nil) {
        self.didAddLogEntryHandler = didAddLogEntryHandler
    }

    public func didAddLogEntry(_ entry: InAppLogEntry) {
        didAddLogEntryHandler?(entry)
    }
}

public struct InAppLogHandler: LogHandler {
    public var metadata: Logger.Metadata = [:]
    public var logLevel: Logger.Level = .debug

    private let process: InAppLogEntry.Process
    private let label: String
    private let observerList = ObserverList<InAppLogBlockObserver>()

    public subscript(metadataKey metadataKey: String) -> Logger.Metadata.Value? {
        get {
            metadata[metadataKey]
        }
        set(newValue) {
            metadata[metadataKey] = newValue
        }
    }

    init(process: InAppLogEntry.Process, label: String, observer: InAppLogBlockObserver) {
        self.process = process
        self.label = label
        observerList.append(observer)
    }

    public func log(
        level: Logger.Level,
        message: Logger.Message,
        metadata: Logger.Metadata?,
        source: String,
        file: String,
        function: String,
        line: UInt
    ) {
        let logEntry = InAppLogEntry(
            process: process,
            timestamp: Date().logFormatted,
            label: label,
            message: message.description
        )

        observerList.notify {
            $0.didAddLogEntry(logEntry)
        }
    }
}
