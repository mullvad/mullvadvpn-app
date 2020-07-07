//
//  RelayCache.swift
//  MullvadVPN
//
//  Created by pronebird on 05/06/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import os

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
    case rpc(MullvadRpc.Error)

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
        case .rpc:
            return "RPC error"
        }
    }
}

protocol RelayCacheObserver: class {
    func relayCache(_ relayCache: RelayCache, didUpdateCachedRelayList cachedRelayList: CachedRelayList)
}

private class AnyRelayCacheObserver: WeakObserverBox, RelayCacheObserver {
    
    typealias Wrapped = RelayCacheObserver

    private(set) weak var inner: RelayCacheObserver?

    init<T: RelayCacheObserver>(_ inner: T) {
        self.inner = inner
    }

    func relayCache(_ relayCache: RelayCache, didUpdateCachedRelayList cachedRelayList: CachedRelayList) {
        inner?.relayCache(relayCache, didUpdateCachedRelayList: cachedRelayList)
    }

    static func == (lhs: AnyRelayCacheObserver, rhs: AnyRelayCacheObserver) -> Bool {
        return lhs.inner === rhs.inner
    }
}

class RelayCache {
    /// Mullvad Rpc client
    private let rpc: MullvadRpc

    /// The cache location used by the class instance
    private let cacheFileURL: URL

    /// A dispatch queue used for thread synchronization
    private let dispatchQueue = DispatchQueue(label: "net.mullvad.MullvadVPN.RelayCache")

    /// A timer source used for periodic updates
    private var timerSource: DispatchSourceTimer?

    /// A flag that indicates whether periodic updates are running
    private var isPeriodicUpdatesEnabled = false

    /// A download task used for relay RPC request
    private var downloadRequest: MullvadRpc.Request<RelayList>?

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
    static let shared = RelayCache(cacheFileURL: defaultCacheFileURL, networkSession: URLSession(configuration: .ephemeral))

    private init(cacheFileURL: URL, networkSession: URLSession) {
        rpc = MullvadRpc(session: networkSession)
        self.cacheFileURL = cacheFileURL
    }

    func startPeriodicUpdates(completionHandler: (() -> Void)?) {
        dispatchQueue.async {
            guard !self.isPeriodicUpdatesEnabled else {
                completionHandler?()
                return
            }

            self.isPeriodicUpdatesEnabled = true

            switch Self.read(cacheFileURL: self.cacheFileURL) {
            case .success(let cachedRelayList):
                if let nextUpdate = Self.nextUpdateDate(lastUpdatedAt: cachedRelayList.updatedAt) {
                    let startTime = Self.makeWalltime(fromDate: nextUpdate)
                    self.scheduleRepeatingTimer(startTime: startTime)
                }

            case .failure(let readError):
                readError.logChain(message: "Failed to read the relay cache")

                if Self.shouldDownloadRelaysOnReadFailure(readError) {
                    self.scheduleRepeatingTimer(startTime: .now())
                }
            }

            completionHandler?()
        }
    }

    func stopPeriodicUpdates(completionHandler: (() -> Void)?) {
        dispatchQueue.async {
            self.isPeriodicUpdatesEnabled = false

            self.timerSource?.cancel()
            self.timerSource = nil
            self.downloadRequest?.cancel()

            completionHandler?()
        }
    }

    func updateRelays() {
        dispatchQueue.async {
            switch Self.read(cacheFileURL: self.cacheFileURL) {
            case .success(let cachedRelays):
                let nextUpdate = Self.nextUpdateDate(lastUpdatedAt: cachedRelays.updatedAt)

                if let nextUpdate = nextUpdate, nextUpdate <= Date() {
                    self.downloadRelays()
                }

            case .failure(let readError):
                readError.logChain(message: "Failed to read the relay cache")

                if Self.shouldDownloadRelaysOnReadFailure(readError) {
                    self.downloadRelays()
                }
            }
        }
    }

    /// Read the relay cache from disk
    func read(completionHandler: @escaping (Result<CachedRelayList, RelayCacheError>) -> Void) {
        dispatchQueue.async {
            let result = Self.read(cacheFileURL: self.cacheFileURL)
                .flatMapError { (error) -> Result<CachedRelayList, RelayCacheError> in
                    if case .readCache(let ioError as CocoaError) = error, ioError.code == .fileReadNoSuchFile {
                        return Self.readPrebundledRelays(fileURL: Self.preBundledRelaysFileURL)
                    } else {
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

    private func downloadRelays() {
        let newDownloadRequest = makeDownloadTask { (result) in
            let result = result.flatMap { (relayList) -> Result<CachedRelayList, RelayCacheError> in
                let cachedRelayList = CachedRelayList(relayList: relayList, updatedAt: Date())

                return Self.write(cacheFileURL: self.cacheFileURL, record: cachedRelayList)
                    .map { cachedRelayList }
            }

            switch result {
            case .success(let cachedRelayList):
                os_log(.default, "Downloaded %d relays", cachedRelayList.relayList.numRelays)

                self.observerList.forEach { (observer) in
                    observer.relayCache(self, didUpdateCachedRelayList: cachedRelayList)
                }

            case .failure(let error):
                error.logChain(message: "Failed to update the relays")
            }
        }

        downloadRequest?.cancel()
        downloadRequest = newDownloadRequest
    }

    private func scheduleRepeatingTimer(startTime: DispatchWallTime) {
        let timerSource = DispatchSource.makeTimerSource()
        timerSource.setEventHandler { [weak self] in
            self?.updateRelays()
        }

        timerSource.schedule(wallDeadline: startTime, repeating: .seconds(kUpdateIntervalSeconds))
        timerSource.activate()

        self.timerSource = timerSource
    }

    private func makeDownloadTask(completionHandler: @escaping (Result<RelayList, RelayCacheError>) -> Void) -> MullvadRpc.Request<RelayList>? {
        let request = rpc.getRelayList()

        request.start { (result) in
            self.dispatchQueue.async {
                let result = result
                    .map(Self.filterRelayList)
                    .mapError { RelayCacheError.rpc($0) }

                completionHandler(result)
            }
        }

        return request
    }

    // MARK: - Private class methods

    /// Filters the given `RelayList` removing empty leaf nodes, relays without Wireguard tunnels or
    /// Wireguard tunnels without any available ports.
    private class func filterRelayList(_ relayList: RelayList) -> RelayList {
        let filteredCountries = relayList.countries
            .map { (country) -> RelayList.Country in
                var filteredCountry = country

                filteredCountry.cities = country.cities.map { (city) -> RelayList.City in
                    var filteredCity = city

                    filteredCity.relays = city.relays
                        .map { (relay) -> RelayList.Relay in
                            var filteredRelay = relay

                            // filter out tunnels without ports
                            filteredRelay.tunnels?.wireguard = relay.tunnels?.wireguard?
                                .filter { !$0.portRanges.isEmpty }

                            return filteredRelay
                    }.filter { $0.tunnels?.wireguard.flatMap { !$0.isEmpty } ?? false }

                    return filteredCity
                }.filter { !$0.relays.isEmpty }

                return filteredCountry
        }.filter { !$0.cities.isEmpty }

        return RelayList(countries: filteredCountries)
    }
    /// Safely read the cache file from disk using file coordinator
    private class func read(cacheFileURL: URL) -> Result<CachedRelayList, RelayCacheError> {
        var result: Result<CachedRelayList, RelayCacheError>?
        let fileCoordinator = NSFileCoordinator(filePresenter: nil)

        let accessor = { (fileURLForReading: URL) -> Void in
            // Decode data from disk
            result = Result { try Data(contentsOf: fileURLForReading) }
                .mapError { RelayCacheError.readCache($0) }
                .flatMap { (data) in
                    Result { try JSONDecoder().decode(CachedRelayList.self, from: data) }
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

    private class func readPrebundledRelays(fileURL: URL) -> Result<CachedRelayList, RelayCacheError> {
        return Result { try Data(contentsOf: fileURL) }
            .mapError { RelayCacheError.readPrebundledRelays($0) }
            .flatMap { (data) -> Result<CachedRelayList, RelayCacheError> in
                return Result { try MullvadRpc.makeJSONDecoder().decode(RelayList.self, from: data) }
                    .mapError { RelayCacheError.decodePrebundledRelays($0) }
                    .map { (relayList) -> CachedRelayList in
                        let now = Date()
                        let updatedAt = Calendar.current.date(byAdding: .second, value: -kUpdateIntervalSeconds, to: now) ?? now

                        return CachedRelayList(relayList: Self.filterRelayList(relayList), updatedAt: updatedAt)
                }
        }
    }

    /// Safely write the cache file on disk using file coordinator
    private class func write(cacheFileURL: URL, record: CachedRelayList) -> Result<(), RelayCacheError> {
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

        case .readCache(let error as CocoaError) where error.code == .fileReadNoSuchFile:
            return true

        default:
            return false
        }
    }
}

/// A struct that represents the relay cache on disk
struct CachedRelayList: Codable {
    /// The relay list stored within the cache entry
    var relayList: RelayList

    /// The date when this cache was last updated
    var updatedAt: Date
}
