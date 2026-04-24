//
//  InAppLogHandler.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

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
        self.observerList.append(observer)
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
        observerList.notify {
            $0.didAddLogEntry(
                InAppLogEntry(
                    process: process,
                    timestamp: Date().logFormatted,
                    label: label,
                    message: message.description
                )
            )
        }
    }
}
