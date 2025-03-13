//
//  RelayCacheTracker.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadREST
import MullvadTypes
import Operations
import UIKit

protocol RelayCacheTrackerProtocol: Sendable {
    func startPeriodicUpdates()
    func stopPeriodicUpdates()
    func updateRelays(completionHandler: ((sending Result<RelaysFetchResult, Error>) -> Void)?) -> Cancellable
    func getCachedRelays() throws -> CachedRelays
    func getNextUpdateDate() -> Date
    func addObserver(_ observer: RelayCacheTrackerObserver)
    func removeObserver(_ observer: RelayCacheTrackerObserver)
    func refreshCachedRelays() throws
}

final class RelayCacheTracker: RelayCacheTrackerProtocol, @unchecked Sendable {
    /// Relay update interval.
    static let relayUpdateInterval: Duration = .seconds(30)

    /// Tracker log.
    nonisolated(unsafe) private let logger = Logger(label: "RelayCacheTracker")

    /// Relay cache.
    private let cache: RelayCacheProtocol

    private let backgroundTaskProvider: BackgroundTaskProviding

    /// Lock used for synchronization.
    private let relayCacheLock = NSLock()

    /// Internal operation queue.
    private let operationQueue = AsyncOperationQueue.makeSerial()

    /// A timer source used for periodic updates.
    private var timerSource: DispatchSourceTimer?

    /// A flag that indicates whether periodic updates are running.
    private var isPeriodicUpdatesEnabled = false

    /// API proxy.
    private let apiProxy: APIQuerying

    /// Observers.
    private let observerList = ObserverList<RelayCacheTrackerObserver>()

    /// Memory cache.
    private var cachedRelays: CachedRelays?

    init(relayCache: RelayCacheProtocol, backgroundTaskProvider: BackgroundTaskProviding, apiProxy: APIQuerying) {
        self.backgroundTaskProvider = backgroundTaskProvider
        self.apiProxy = apiProxy
        cache = relayCache

        do {
            cachedRelays = try cache.read().cachedRelays
            try hotfixRelaysThatDoNotHaveDaita()
        } catch {
            logger.error(
                error: error,
                message: "Failed to read the relay cache during initialization."
            )

            _ = updateRelays(completionHandler: nil)
        }
    }

    /// This method updates the cached relay to include daita information
    ///
    /// This is a hotfix meant to upgrade clients shipped with 2024.5 or before that did not have
    /// daita information in their representation of `ServerRelay`.
    /// If a version <= 2024.5 is installed less than an hour before a new upgrade,
    /// no servers will be shown in locations when filtering for daita relays.
    ///
    /// > Info: `relayCacheLock` does not need to be accessed here, this method should be ran from `init` only.
    private func hotfixRelaysThatDoNotHaveDaita() throws {
        guard let cachedRelays else { return }
        let daitaPropertyMissing = cachedRelays.relays.wireguard.relays.first { $0.daita ?? false } == nil
        // If the cached relays already have daita information, this fix is not necessary
        guard daitaPropertyMissing else { return }

        let preBundledRelays = try cache.readPrebundledRelays().relays
        let preBundledDaitaRelays = preBundledRelays.wireguard.relays.filter { $0.daita == true }
        var cachedRelaysWithFixedDaita = cachedRelays.relays.wireguard.relays

        // For each daita enabled relay in the prebundled relays
        // Find the corresponding relay in the cache by matching relay hostnames
        // Then update it to toggle daita
        for index in 0 ..< cachedRelaysWithFixedDaita.endIndex {
            let relay = cachedRelaysWithFixedDaita[index]
            preBundledDaitaRelays.forEach {
                if $0.hostname == relay.hostname {
                    cachedRelaysWithFixedDaita[index] = relay.override(daita: true)
                }
            }
        }

        let wireguard = REST.ServerWireguardTunnels(
            ipv4Gateway:
            cachedRelays.relays.wireguard.ipv4Gateway,
            ipv6Gateway: cachedRelays.relays.wireguard.ipv6Gateway,
            portRanges: cachedRelays.relays.wireguard.portRanges,
            relays: cachedRelaysWithFixedDaita,
            shadowsocksPortRanges: cachedRelays.relays.wireguard.shadowsocksPortRanges
        )

        let updatedRelays = REST.ServerRelaysResponse(
            locations: cachedRelays.relays.locations,
            wireguard: wireguard,
            bridge: cachedRelays.relays.bridge
        )

        let updatedRawRelayData = try REST.Coding.makeJSONEncoder().encode(updatedRelays)
        let updatedCachedRelays = try StoredRelays(
            etag: cachedRelays.etag,
            rawData: updatedRawRelayData,
            updatedAt: cachedRelays.updatedAt
        )

        try cache.write(record: updatedCachedRelays)
        self.cachedRelays = CachedRelays(
            etag: cachedRelays.etag,
            relays: updatedRelays,
            updatedAt: cachedRelays.updatedAt
        )
    }

    func startPeriodicUpdates() {
        relayCacheLock.lock()
        defer { relayCacheLock.unlock() }

        guard !isPeriodicUpdatesEnabled else { return }

        logger.debug("Start periodic relay updates.")

        isPeriodicUpdatesEnabled = true

        let nextUpdate = _getNextUpdateDate()

        scheduleRepeatingTimer(startTime: .now() + nextUpdate.timeIntervalSinceNow)
    }

    func stopPeriodicUpdates() {
        relayCacheLock.lock()
        defer { relayCacheLock.unlock() }

        guard isPeriodicUpdatesEnabled else { return }

        logger.debug("Stop periodic relay updates.")

        isPeriodicUpdatesEnabled = false

        timerSource?.cancel()
        timerSource = nil
    }

    func updateRelays(completionHandler: ((sending Result<RelaysFetchResult, Error>) -> Void)? = nil)
        -> Cancellable {
        let operation = ResultBlockOperation<RelaysFetchResult> { finish in
            let cachedRelays = try? self.getCachedRelays()

            if self.getNextUpdateDate() > Date() {
                finish(.success(.throttled))
                return AnyCancellable()
            }

            return self.apiProxy.getRelays(etag: "hello", retryStrategy: .noRetry) { result in
                print(result)
                finish(self.handleResponse(result: result))
            }

//            return self.apiProxy.mullvadApiGetRelayList(retryStrategy: .noRetry, etag: cachedRelays.etag) { result in
//                finish(self.handleResponse(result: result))
//            }
        }

        operation.addObserver(
            BackgroundObserver(
                backgroundTaskProvider: backgroundTaskProvider,
                name: "Update relays",
                cancelUponExpiration: true
            )
        )

        operation.completionQueue = .main
        operation.completionHandler = completionHandler

        operationQueue.addOperation(operation)

        return operation
    }

    func getCachedRelays() throws -> CachedRelays {
        relayCacheLock.lock()
        defer { relayCacheLock.unlock() }

        if let cachedRelays {
            return cachedRelays
        } else {
            throw NoCachedRelaysError()
        }
    }

    func refreshCachedRelays() throws {
        let newCachedRelays = try cache.read().cachedRelays

        relayCacheLock.lock()
        cachedRelays = newCachedRelays
        relayCacheLock.unlock()

        DispatchQueue.main.async {
            self.observerList.notify { observer in
                observer.relayCacheTracker(self, didUpdateCachedRelays: newCachedRelays)
            }
        }
    }

    func getNextUpdateDate() -> Date {
        relayCacheLock.lock()
        defer { relayCacheLock.unlock() }

        return _getNextUpdateDate()
    }

    // MARK: - Observation

    func addObserver(_ observer: RelayCacheTrackerObserver) {
        observerList.append(observer)
    }

    func removeObserver(_ observer: RelayCacheTrackerObserver) {
        observerList.remove(observer)
    }

    // MARK: - Private

    private func _getNextUpdateDate() -> Date {
        let now = Date()

        guard let cachedRelays else {
            return now
        }

        let nextUpdate = cachedRelays.updatedAt.addingTimeInterval(Self.relayUpdateInterval.timeInterval)

        return max(nextUpdate, Date())
    }

    private func handleResponse(result: Result<REST.ServerRelaysCacheResponse, Error>)
        -> Result<RelaysFetchResult, Error> {
        result.tryMap { response -> RelaysFetchResult in
            switch response {
            case let .newContent(etag, rawData):
                try self.storeResponse(etag: etag, rawData: rawData)

                return .newContent

            case .notModified:
                return .sameContent
            }
        }.inspectError { error in
            guard !error.isOperationCancellationError else { return }

            logger.error(
                error: error,
                message: "Failed to update relays."
            )
        }
    }

    private func storeResponse(etag: String?, rawData: Data) throws {
        let newCachedData = try StoredRelays(
            etag: etag,
            rawData: rawData,
            updatedAt: Date()
        )

        try cache.write(record: newCachedData)
        try refreshCachedRelays()
    }

    private func scheduleRepeatingTimer(startTime: DispatchWallTime) {
        let timerSource = DispatchSource.makeTimerSource()
        timerSource.setEventHandler { [weak self] in
            _ = self?.updateRelays()
        }

        timerSource.schedule(
            wallDeadline: startTime,
            repeating: Self.relayUpdateInterval.timeInterval
        )
        timerSource.activate()

        self.timerSource = timerSource
    }
}

/// Type describing the result of an attempt to fetch the new relay list from server.
enum RelaysFetchResult: CustomStringConvertible {
    /// Request to update relays was throttled.
    case throttled

    /// Refreshed relays but the same content was found on remote.
    case sameContent

    /// Refreshed relays with new content.
    case newContent

    var description: String {
        switch self {
        case .throttled:
            return "throttled"
        case .sameContent:
            return "same content"
        case .newContent:
            return "new content"
        }
    }
}

struct NoCachedRelaysError: LocalizedError {
    var errorDescription: String? {
        "Relay cache is empty."
    }
}
