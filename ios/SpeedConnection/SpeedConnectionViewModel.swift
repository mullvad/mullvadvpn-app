//
//  SpeedConnectionViewModel.swift
//  MullvadVPN
//
//  Created by Mojgan on 2026-04-23.
//  Copyright © 2026 Mullvad VPN AB. All rights reserved.
//

import Combine

protocol SpeedConnectionViewModelProtocol: ObservableObject {
    var downloadText: String { get }
    var uploadText: String { get }

    func startMonitoring()
    func stopMonitoring()
}

class SpeedConnectionViewModel: SpeedConnectionViewModelProtocol, @unchecked Sendable {
    @Published var downloadText: String = ""
    @Published var uploadText: String = ""
    var networkSpeedMonitor: NetworkSpeedMonitorProtocol

    init(networkSpeedMonitor: NetworkSpeedMonitorProtocol) {
        self.networkSpeedMonitor = networkSpeedMonitor
        self.networkSpeedMonitor.onUpdateTrafficSummery = { [weak self] trafficSummery in
            Task { @MainActor in
                self?.update(trafficSummery.speed)
            }
        }
    }
    func startMonitoring() {
        self.networkSpeedMonitor.start(timeInterval: 1.0)
    }

    func stopMonitoring() {
        self.networkSpeedMonitor.stop()
    }

    private func update(_ speed: TrafficSpeed) {
        let down = SpeedFormatter.display(speed.received)
        let up = SpeedFormatter.display(speed.sent)

        downloadText = "Download: \(down.value.formatted(.number.precision(.fractionLength(2)))) \(down.unit)"
        uploadText = "Upload: \(up.value.formatted(.number.precision(.fractionLength(2)))) \(up.unit)"

    }
}

private struct SpeedFormatter {
    static func display(_ bps: Double) -> (value: Double, unit: String) {
        if bps >= 1_000_000_000 {
            return (bps / 1_000_000_000, "Gbps")
        } else if bps >= 1_000_000 {
            return (bps / 1_000_000, "Mbps")
        } else {
            return (bps / 1_000, "Kbps")
        }
    }
}
