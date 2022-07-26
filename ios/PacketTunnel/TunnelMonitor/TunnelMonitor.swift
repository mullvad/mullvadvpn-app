//
//  TunnelMonitor.swift
//  PacketTunnel
//
//  Created by pronebird on 09/02/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import NetworkExtension
import WireGuardKit
import Logging

final class TunnelMonitor {
    private let adapter: WireGuardAdapter
    private let internalQueue = DispatchQueue(label: "TunnelMonitor")
    private let delegateQueue: DispatchQueue

    private var address: IPv4Address?
    private var pinger: Pinger?
    private var pathMonitor: NWPathMonitor?
    private var networkBytesReceived: UInt64 = 0
    private var firstAttemptDate: Date?
    private var lastAttemptDate: Date?
    private var lastError: Pinger.Error?
    private var isStarted = false
    private var isPinging = false

    private var logger = Logger(label: "TunnelMonitor")
    private var timer: DispatchSourceTimer?

    private weak var _delegate: TunnelMonitorDelegate?
    weak var delegate: TunnelMonitorDelegate? {
        set {
            internalQueue.sync {
                _delegate = newValue
            }
        }
        get {
            return internalQueue.sync {
                return _delegate
            }
        }
    }

    var startDate: Date? {
        return internalQueue.sync {
            return firstAttemptDate
        }
    }

    init(queue: DispatchQueue, adapter: WireGuardAdapter) {
        delegateQueue = queue
        self.adapter = adapter
    }

    deinit {
        stopNoQueue(forRestart: false)
    }

    func start(address: IPv4Address) {
        internalQueue.async {
            self.startNoQueue(address: address)
        }
    }

    func stop() {
        internalQueue.async {
            self.stopNoQueue(forRestart: false)
        }
    }

    private func startNoQueue(address pingAddress: IPv4Address) {
        if isStarted {
            logger.debug("Restart tunnel monitor with address: \(pingAddress).")
        } else {
            logger.debug("Start tunnel monitor with address: \(pingAddress).")
        }

        stopNoQueue(forRestart: true)

        isStarted = true
        address = pingAddress
        networkBytesReceived = 0
        firstAttemptDate = Date()
        lastAttemptDate = firstAttemptDate
        lastError = nil

        let newPathMonitor = NWPathMonitor()
        newPathMonitor.pathUpdateHandler = { [weak self] path in
            self?.handleNetworkPathUpdate(path)
        }
        newPathMonitor.start(queue: internalQueue)
        pathMonitor = newPathMonitor

        handleNetworkPathUpdate(newPathMonitor.currentPath)
    }

    private func stopNoQueue(forRestart: Bool) {
        if isStarted && !forRestart {
            logger.debug("Stop tunnel monitor.")
        }

        isStarted = false
        address = nil
        firstAttemptDate = nil
        lastAttemptDate = nil
        lastError = nil

        pathMonitor?.cancel()
        pathMonitor = nil

        cancelWgStatsTimer()
        stopPinging()
    }

    private func startPinging(address: IPv4Address) throws {
        let newPinger = Pinger(address: address, interfaceName: adapter.interfaceName)

        try newPinger.start(
            delay: TunnelMonitorConfiguration.pingStartDelay,
            repeating: TunnelMonitorConfiguration.pingInterval
        )

        pinger = newPinger
        isPinging = true
    }

    private func stopPinging() {
        pinger?.stop()
        pinger = nil

        isPinging = false
    }

    private func setWgStatsTimer() {
        // Cancel existing timer.
        cancelWgStatsTimer()

        // Create new timer.
        timer = DispatchSource.makeTimerSource(queue: internalQueue)
        timer?.setEventHandler { [weak self] in
            self?.onWgStatsTimer()
        }
        timer?.schedule(wallDeadline: .now(), repeating: TunnelMonitorConfiguration.wgStatsQueryInterval)
        timer?.resume()

        logger.debug("Set WG stats timer.")
    }

    private func cancelWgStatsTimer() {
        timer?.cancel()
        timer = nil
    }

    private func onWgStatsTimer() {
        adapter.getRuntimeConfiguration { [weak self] str in
            guard let self = self else { return }

            self.internalQueue.async {
                self.handleWgStatsUpdate(string: str)
            }
        }
    }

    private func handleWgStatsUpdate(string: String?) {
        guard let string = string else {
            logger.debug("Received no runtime configuration from WireGuard adapter.")
            return
        }

        guard let newNetworkBytesReceived = Self.parseNetworkBytesReceived(from: string) else {
            logger.debug("Failed to parse rx_bytes from runtime configuration.")
            return
        }

        let oldNetworkBytesReceived = self.networkBytesReceived
        networkBytesReceived = newNetworkBytesReceived

        if newNetworkBytesReceived < oldNetworkBytesReceived {
            logger.debug("Stats was reset? newNetworkBytesReceived = \(newNetworkBytesReceived), oldNetworkBytesReceived = \(oldNetworkBytesReceived)")
            return
        }

        if newNetworkBytesReceived > oldNetworkBytesReceived {
            // Tell delegate that connection is established.
            delegateQueue.async {
                self.delegate?.tunnelMonitorDidDetermineConnectionEstablished(self)
            }

            // Stop the tunnel monitor.
            stopNoQueue(forRestart: false)

            return
        }

        if let nextAttemptDate = lastAttemptDate?.addingTimeInterval(TunnelMonitorConfiguration.connectionTimeout), nextAttemptDate <= Date() {
            // Reset the last recovery attempt date.
            lastAttemptDate = nextAttemptDate

            // Reset last error.
            lastError = nil

            // Tell delegate to attempt the connection recovery.
            delegateQueue.async {
                self.delegate?.tunnelMonitorDelegateShouldHandleConnectionRecovery(self)
            }
        }
    }

    private func handleNetworkPathUpdate(_ networkPath: Network.NWPath) {
        guard let address = address else {
            return
        }

        let isNetworkReachable = isNetworkPathReachable(networkPath)

        switch (isNetworkReachable, isPinging) {
        case (true, false):
            logger.debug("Network is reachable. Starting to ping.")

            do {
                try startPinging(address: address)

                // Reset the last recovery attempt date.
                firstAttemptDate = Date()
                lastAttemptDate = firstAttemptDate

                // Start WG stats timer.
                setWgStatsTimer()

                delegateQueue.async {
                    self.delegate?.tunnelMonitor(self, networkReachabilityStatusDidChange: isNetworkReachable)
                }
            } catch {
                let error = error as! Pinger.Error

                if error != lastError {
                    logger.error(
                        chainedError: AnyChainedError(error),
                        message: "Failed to start pinging."
                    )
                    lastError = error
                }
            }

        case (false, true):
            logger.debug("Network is unreachable. Stop pinging and wait...")

            // Cancel timers and ping.
            cancelWgStatsTimer()
            stopPinging()

            // Reset the last recovery attempt date.
            lastAttemptDate = nil

            delegateQueue.async {
                self.delegate?.tunnelMonitor(self, networkReachabilityStatusDidChange: isNetworkReachable)
            }

        default:
            break
        }
    }

    private func isNetworkPathReachable(_ networkPath: Network.NWPath) -> Bool {
        // Get utun interface name.
        guard let tunName = adapter.interfaceName else { return false }

        // Check if utun is up.
        let utunUp = networkPath.availableInterfaces.contains { interface in
            return interface.name == tunName
        }

        // Return false if tunnel is down.
        guard utunUp else {
            return false
        }

        // Return false if utun is the only available interface.
        if networkPath.availableInterfaces.count == 1 {
            return false
        }

        switch networkPath.status {
        case .requiresConnection, .satisfied:
            return true
        case .unsatisfied:
            return false
        @unknown default:
            return false
        }
    }

    private class func parseNetworkBytesReceived(from string: String) -> UInt64? {
        guard let range = string.range(of: "rx_bytes=") else { return nil }

        let startIndex = range.upperBound
        let endIndex = string[startIndex...].firstIndex { ch in
            return ch.isNewline
        }

        if let endIndex = endIndex {
            return UInt64(string[startIndex..<endIndex])
        } else {
            return nil
        }
    }
}
