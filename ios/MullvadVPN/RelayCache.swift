//
//  RelayCache.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import Logging

/// Periodic update interval
private let kUpdateIntervalSeconds = 3600

/// Error emitted by read and write functions
enum RelayCacheError: ChainedError {
    case readCache(Error)
    case readPrebundledRelays(Error)
    case decodePrebundledRelays(Error)
    case writeCache(Error)
    case encodeCache(Error)
    case decodeCache(Error)
    case rest(RestError)

    var errorDescription: String? {
        switch self {
        case .encodeCache:
            return "Encode cache error"
        case .decodeCache:
            return "Decode cache error"
        case .readCache:
            return "Read cache error"
        case .readPrebundledRelays:
            return "Read pre-bundled relays error"
        case .decodePrebundledRelays:
            return "Decode pre-bundled relays error"
        case .writeCache:
            return "Write cache error"
        case .rest:
            return "REST error"
        }
    }
}

protocol RelayCacheObserver: class {
    func relayCache(_ relayCache: RelayCache, didUpdateCachedRelays cachedRelays: CachedRelays)
}

private class AnyRelayCacheObserver: WeakObserverBox, RelayCacheObserver {

    typealias Wrapped = RelayCacheObserver

    private(set) weak var inner: RelayCacheObserver?

    init<T: RelayCacheObserver>(_ inner: T) {
        self.inner = inner
    }

    func relayCache(_ relayCache: RelayCache, didUpdateCachedRelays cachedRelays: CachedRelays) {
        inner?.relayCache(relayCache, didUpdateCachedRelays: cachedRelays)
    }

    static func == (lhs: AnyRelayCacheObserver, rhs: AnyRelayCacheObserver) -> Bool {
        return lhs.inner === rhs.inner
    }
}

class RelayCache {
    private let logger = Logger(label: "RelayCache")

    /// Mullvad REST client
    private let rest = MullvadRest()

    /// The cache location used by the class instance
    private let cacheFileURL: URL

    /// A dispatch queue used for thread synchronization
    private let dispatchQueue = DispatchQueue(label: "net.mullvad.MullvadVPN.RelayCache")

    /// A timer source used for periodic updates
    private var timerSource: DispatchSourceTimer?

    /// A flag that indicates whether periodic updates are running
    private var isPeriodicUpdatesEnabled = false

    /// A download task used for relay RPC request
    private var downloadTask: URLSessionTask?

    /// The default cache file location
    static var defaultCacheFileURL: URL {
        let appGroupIdentifier = ApplicationConfiguration.securityGroupIdentifier
        let containerURL = FileManager.default.containerURL(forSecurityApplicationGroupIdentifier: appGroupIdentifier)!

        return containerURL.appendingPathComponent("relays.json")
    }

    /// The path to the pre-bundled relays.json file
    private static var preBundledRelaysFileURL: URL {
        return Bundle.main.url(forResource: "relays", withExtension: "json")!
    }

    /// Observers
    private let observerList = ObserverList<AnyRelayCacheObserver>()

    /// A shared instance of `RelayCache`
    static let shared = RelayCache(cacheFileURL: defaultCacheFileURL)

    private init(cacheFileURL: URL) {
        self.cacheFileURL = cacheFileURL
    }

    func startPeriodicUpdates(queue: DispatchQueue?, completionHandler: (() -> Void)?) {
        dispatchQueue.async {
            if !self.isPeriodicUpdatesEnabled {
                self.isPeriodicUpdatesEnabled = true

                switch Self.read(cacheFileURL: self.cacheFileURL) {
                case .success(let cachedRelayList):
                    if let nextUpdate = Self.nextUpdateDate(lastUpdatedAt: cachedRelayList.updatedAt) {
                        let startTime = Self.makeWalltime(fromDate: nextUpdate)
                        self.scheduleRepeatingTimer(startTime: startTime)
                    }

                case .failure(let readError):
                    self.logger.error(chainedError: readError, message: "Failed to read the relay cache")

                    if Self.shouldDownloadRelaysOnReadFailure(readError) {
                        self.scheduleRepeatingTimer(startTime: .now())
                    }
                }
            }

            queue.performOnWrappedOrCurrentQueue {
                completionHandler?()
            }
        }
    }

    func stopPeriodicUpdates(queue: DispatchQueue?, completionHandler: (() -> Void)?) {
        dispatchQueue.async {
            self.isPeriodicUpdatesEnabled = false

            self.timerSource?.cancel()
            self.timerSource = nil
            self.downloadTask?.cancel()

            queue.performOnWrappedOrCurrentQueue {
                completionHandler?()
            }
        }
    }

    func updateRelays() {
        dispatchQueue.async {
            self._updateRelays()
        }
    }

    /// Read the relay cache from disk
    func read(completionHandler: @escaping (Result<CachedRelays, RelayCacheError>) -> Void) {
        dispatchQueue.async {
            let result = Self.read(cacheFileURL: self.cacheFileURL)
                .flatMapError { (error) -> Result<CachedRelays, RelayCacheError> in
                    switch error {
                    case .decodeCache, .readCache(CocoaError.fileReadNoSuchFile):
                        return Self.readPrebundledRelays(fileURL: Self.preBundledRelaysFileURL)
                    default:
                        return .failure(error)
                    }
            }
            completionHandler(result)
        }
    }

    // MARK: - Observation

    func addObserver<T: RelayCacheObserver>(_ observer: T) {
        observerList.append(AnyRelayCacheObserver(observer))
    }

    func removeObserver<T: RelayCacheObserver>(_ observer: T) {
        observerList.remove(AnyRelayCacheObserver(observer))
    }

    // MARK: - Private instance methods

    private func _updateRelays() {
        switch Self.read(cacheFileURL: self.cacheFileURL) {
        case .success(let cachedRelays):
            let nextUpdate = Self.nextUpdateDate(lastUpdatedAt: cachedRelays.updatedAt)

            if let nextUpdate = nextUpdate, nextUpdate <= Date() {
                self.downloadRelays(previouslyCachedRelays: cachedRelays)
            }

        case .failure(let readError):
            self.logger.error(chainedError: readError, message: "Failed to read the relay cache to determine if it needs to be updated")

            if Self.shouldDownloadRelaysOnReadFailure(readError) {
                self.downloadRelays(previouslyCachedRelays: nil)
            }
        }
    }

    private func downloadRelays(previouslyCachedRelays: CachedRelays?) {
        let taskResult = makeDownloadTask(etag: previouslyCachedRelays?.etag) { (result) in
            switch result {
            case .success(.newContent(let etag, let relays)):
                let numRelays = relays.wireguard.relays.count

                self.logger.info("Downloaded \(numRelays) relays")

                let cachedRelays = CachedRelays(etag: etag, relays: relays, updatedAt: Date())
                switch Self.write(cacheFileURL: self.cacheFileURL, record: cachedRelays) {
                case .success:
                    self.observerList.forEach { (observer) in
                        observer.relayCache(self, didUpdateCachedRelays: cachedRelays)
                    }

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to store downloaded relays")
                }

            case .success(.notModified):
                self.logger.info("Relays haven't changed since last check.")

                var cachedRelays = previouslyCachedRelays!
                cachedRelays.updatedAt = Date()

                switch Self.write(cacheFileURL: self.cacheFileURL, record: cachedRelays) {
                case .success:
                    break

                case .failure(let error):
                    self.logger.error(chainedError: error, message: "Failed to update cached relays timestamp")
                }

            case .failure(let error):
                self.logger.error(chainedError: error, message: "Failed to download relays")
            }
        }

        downloadTask?.cancel()

        switch taskResult {
        case .success(let newDownloadTask):
            downloadTask = newDownloadTask
            newDownloadTask.resume()

        case .failure(let restError):
            self.logger.error(chainedError: restError, message: "Failed to create a REST request for updating relays")
            downloadTask = nil
        }
    }

    private func scheduleRepeatingTimer(startTime: DispatchWallTime) {
        let timerSource = DispatchSource.makeTimerSource(queue: dispatchQueue)
        timerSource.setEventHandler { [weak self] in
            guard let self = self else { return }

            if self.isPeriodicUpdatesEnabled {
                self._updateRelays()
            }
        }

        timerSource.schedule(wallDeadline: startTime, repeating: .seconds(kUpdateIntervalSeconds))
        timerSource.activate()

        self.timerSource = timerSource
    }

    private func makeDownloadTask(etag: String?, completionHandler: @escaping (Result<HttpResourceCacheResponse<ServerRelaysResponse>, RelayCacheError>) -> Void) -> Result<URLSessionDataTask, RestError> {
        return rest.getRelays().dataTask(payload: ETagPayload(etag: etag, enforceWeakValidator: true, payload: EmptyPayload())) { (result) in
            self.dispatchQueue.async {
                completionHandler(result.mapError { RelayCacheError.rest($0) })
            }
        }
    }

    // MARK: - Private class methods

    /// Safely read the cache file from disk using file coordinator
    private class func read(cacheFileURL: URL) -> Result<CachedRelays, RelayCacheError> {
        var result: Result<CachedRelays, RelayCacheError>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForReading: URL) -> Void in
            // Decode data from disk
            result = Result { try Data(contentsOf: fileURLForReading) }
                .mapError { RelayCacheError.readCache($0) }
                .flatMap { (data) in
                    Result { try JSONDecoder().decode(CachedRelays.self, from: data) }
                        .mapError { RelayCacheError.decodeCache($0) }
                }
        }

        var error: NSError?
        fileCoordinator.coordinate(readingItemAt: cacheFileURL,
                                   options: [.withoutChanges],
                                   error: &error,
                                   byAccessor: accessor)

        if let error = error {
            result = .failure(.readCache(error))
        }

        return result!
    }

    private class func readPrebundledRelays(fileURL: URL) -> Result<CachedRelays, RelayCacheError> {
        return Result { try Data(contentsOf: fileURL) }
            .mapError { RelayCacheError.readPrebundledRelays($0) }
            .flatMap { (data) -> Result<CachedRelays, RelayCacheError> in
                return Result { try MullvadRest.makeJSONDecoder().decode(ServerRelaysResponse.self, from: data) }
                    .mapError { RelayCacheError.decodePrebundledRelays($0) }
                    .map { (relays) -> CachedRelays in
                        return CachedRelays(
                            relays: relays,
                            updatedAt: Date(timeIntervalSince1970: 0)
                        )
                }
        }
    }

    /// Safely write the cache file on disk using file coordinator
    private class func write(cacheFileURL: URL, record: CachedRelays) -> Result<(), RelayCacheError> {
        var result: Result<(), RelayCacheError>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForWriting: URL) -> Void in
            result = Result { try JSONEncoder().encode(record) }
                .mapError { RelayCacheError.encodeCache($0) }
                .flatMap { (data) in
                    Result { try data.write(to: fileURLForWriting) }
                        .mapError { RelayCacheError.writeCache($0) }
                }
        }

        var error: NSError?
        fileCoordinator.coordinate(writingItemAt: cacheFileURL,
                                   options: [.forReplacing],
                                   error: &error,
                                   byAccessor: accessor)

        if let error = error {
            result = .failure(.writeCache(error))
        }

        return result!
    }

    private class func makeWalltime(fromDate date: Date) -> DispatchWallTime {
        let (seconds, frac) = modf(date.timeIntervalSince1970)

        let nsec: Double = frac * Double(NSEC_PER_SEC)
        let walltime = timespec(tv_sec: Int(seconds), tv_nsec: Int(nsec))

        return DispatchWallTime(timespec: walltime)
    }

    private class func nextUpdateDate(lastUpdatedAt: Date) -> Date? {
        return Calendar.current.date(
            byAdding: .second,
            value: kUpdateIntervalSeconds,
            to: lastUpdatedAt
        )
    }

    private class func shouldDownloadRelaysOnReadFailure(_ error: RelayCacheError) -> Bool {
        switch error {
        case .readPrebundledRelays, .decodePrebundledRelays, .decodeCache:
            return true

        case .readCache(CocoaError.fileReadNoSuchFile):
            return true

        default:
            return false
        }
    }
}

/// A struct that represents the relay cache on disk
struct CachedRelays: Codable {
    /// E-tag returned by server
    var etag: String?

    /// The relay list stored within the cache entry
    var relays: ServerRelaysResponse

    /// The date when this cache was last updated
    var updatedAt: Date
}
