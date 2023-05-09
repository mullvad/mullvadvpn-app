//
//  TunnelBlockObserver.swift
//  MullvadVPN
//
//  Created by pronebird on 26/10/2022.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation

final class TunnelBlockObserver: TunnelObserver {
    typealias DidLoadConfigurationHandler = (TunnelManager) -> Void
    typealias DidUpdateTunnelStatusHandler = (TunnelManager, TunnelStatus) -> Void
    typealias DidUpdateDeviceStateHandler = (
        _ tunnelManager: TunnelManager,
        _ deviceState: DeviceState,
        _ previousDeviceState: DeviceState
    ) -> Void
    typealias DidUpdateTunnelSettingsHandler = (TunnelManager, TunnelSettingsV2) -> Void
    typealias DidFailWithErrorHandler = (TunnelManager, Error) -> Void

    private let didLoadConfiguration: DidLoadConfigurationHandler?
    private let didUpdateTunnelStatus: DidUpdateTunnelStatusHandler?
    private let didUpdateDeviceState: DidUpdateDeviceStateHandler?
    private let didUpdateTunnelSettings: DidUpdateTunnelSettingsHandler?
    private let didFailWithError: DidFailWithErrorHandler?

    init(
        didLoadConfiguration: DidLoadConfigurationHandler? = nil,
        didUpdateTunnelStatus: DidUpdateTunnelStatusHandler? = nil,
        didUpdateDeviceState: DidUpdateDeviceStateHandler? = nil,
        didUpdateTunnelSettings: DidUpdateTunnelSettingsHandler? = nil,
        didFailWithError: DidFailWithErrorHandler? = nil
    ) {
        self.didLoadConfiguration = didLoadConfiguration
        self.didUpdateTunnelStatus = didUpdateTunnelStatus
        self.didUpdateDeviceState = didUpdateDeviceState
        self.didUpdateTunnelSettings = didUpdateTunnelSettings
        self.didFailWithError = didFailWithError
    }

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {
        didLoadConfiguration?(manager)
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelStatus tunnelStatus: TunnelStatus) {
        didUpdateTunnelStatus?(manager, tunnelStatus)
    }

    func tunnelManager(
        _ manager: TunnelManager,
        didUpdateDeviceState deviceState: DeviceState,
        previousDeviceState: DeviceState
    ) {
        didUpdateDeviceState?(manager, deviceState, previousDeviceState)
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2) {
        didUpdateTunnelSettings?(manager, tunnelSettings)
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {
        didFailWithError?(manager, error)
    }
}
