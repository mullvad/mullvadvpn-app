//
//  InAppLogHandler.swift
//  MullvadVPN
//
//  Created by Jon Petersson on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes

public protocol InAppLogObserver: AnyObject {
    func didAddLogEntry(_ entry: String)
}

public final class InAppLogBlockObserver: InAppLogObserver, @unchecked Sendable {
    public typealias DidAddLogEntryHandler = (String) -> Void

    public var didAddLogEntryHandler: DidAddLogEntryHandler?

    public init(didAddLogEntryHandler: DidAddLogEntryHandler? = nil) {
        self.didAddLogEntryHandler = didAddLogEntryHandler
    }

    public func didAddLogEntry(_ entry: String) {
        DispatchQueue.main.async { [weak self] in
            self?.didAddLogEntryHandler?(entry)
        }
    }
}

public struct InAppLogHandler: LogHandler {
    public var metadata: Logger.Metadata = [:]
    public var logLevel: Logger.Level = .debug

    private let label: String
    private let observerList = ObserverList<InAppLogObserver>()

    private struct RegistryKey: Hashable {
        let subsystem: String
        let category: String
    }

    public subscript(metadataKey metadataKey: String) -> Logger.Metadata.Value? {
        get {
            metadata[metadataKey]
        }
        set(newValue) {
            metadata[metadataKey] = newValue
        }
    }

    init(label: String, observer: InAppLogObserver) {
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
        let mergedMetadata = self.metadata
            .merging(metadata ?? [:]) { _, rhs -> Logger.MetadataValue in
                rhs
            }
        let prettyMetadata = Self.formatMetadata(mergedMetadata)
        let metadataOutput = prettyMetadata.isEmpty ? "" : " \(prettyMetadata)"
        let timestamp = Date().logFormatted
        let formattedMessage = "[\(timestamp)][\(label)]\n\(message)\n"

        observerList.notify {
            $0.didAddLogEntry(formattedMessage)
        }
    }

    private static func formatMetadata(_ metadata: Logger.Metadata) -> String {
        metadata.map { "\($0)=\($1)" }.joined(separator: " ")
    }
}
