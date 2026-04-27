//
//  NetworkSpeedMonitor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2025-10-31.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//
import Foundation
import SystemConfiguration

protocol NetworkSpeedMonitorProtocol {
    var onUpdateTrafficSummery: (@Sendable (TrafficSummery) -> Void)? { get set }
    func start(timeInterval: TimeInterval)
    func stop()
}

final class NetworkSpeedMonitor: NetworkSpeedMonitorProtocol {
    private var timer: DispatchSourceTimer? = nil
    private var previousTrafficPackage = TrafficPackage()
    private var lock = NSLock()
    private let timerQueue = DispatchQueue(label: "NetworkSpeedMonitorTimerQueue")

    var onUpdateTrafficSummery: (@Sendable (TrafficSummery) -> Void)?

    func start(timeInterval: TimeInterval = 1.0) {
        timer?.cancel()
        timer = DispatchSource.makeTimerSource(queue: timerQueue)
        timer?.setEventHandler { [weak self] in
            guard let self = self else { return }
            self.measure()
        }
        timer?.schedule(wallDeadline: .now(), repeating: timeInterval)
        timer?.resume()
    }

    func stop() {
        timer?.cancel()
        timer = nil
    }

    private func measure() {
        lock.lock()
        defer {
            lock.unlock()
        }
        let newTrafficPacket = getTrafficPackage()
        onUpdateTrafficSummery?(TrafficSummery.make(previousTrafficPackage, new: newTrafficPacket, interval: 1))
        previousTrafficPackage = newTrafficPacket
    }

    private func getTrafficPackage() -> TrafficPackage {
        var result = TrafficPackage()
        var address: UnsafeMutablePointer<ifaddrs>? = nil

        guard getifaddrs(&address) == 0, let first = address else {
            return result
        }

        defer {
            freeifaddrs(first)
        }

        var pointer: UnsafeMutablePointer<ifaddrs>? = first

        while let current = pointer?.pointee {

            defer {
                pointer = current.ifa_next
            }

            let interfaceName = String(cString: current.ifa_name)
            let interface = mapInterface(interfaceName)

            guard
                let dataPtr = current.ifa_data
            else {
                continue
            }

            let data = dataPtr.assumingMemoryBound(to: if_data.self).pointee

            let sent = UInt64(data.ifi_obytes)
            let received = UInt64(data.ifi_ibytes)

            switch interface {
            case .cellular:
                result.cellular.sent += sent
                result.cellular.received += received

            case .wifi:
                result.wifi.sent += sent
                result.wifi.received += received

            case .vpn:
                result.vpn.sent += sent
                result.vpn.received += received

            default:
                break
            }
        }

        return result
    }

    private func mapInterface(_ name: String) -> InterfaceType {
        if name.hasPrefix("en") {
            return .wifi
        } else if name.hasPrefix("pdp_ip") {
            return .cellular
        } else if name.hasPrefix("utun") || name.hasPrefix("ppp") {
            return .vpn
        } else if name == "lo0" {
            return .loopback
        } else {
            return .other
        }
    }
}
