//
//  WelcomeInteractor.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-06-29.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import Foundation
import MullvadLogging
import MullvadTypes
import StoreKit

final class WelcomeInteractor: @unchecked Sendable {
    private let storePaymentManager: StorePaymentManager
    private let tunnelManager: TunnelManager

    /// Interval used for periodic polling account updates.
    private let accountUpdateTimerInterval: Duration = .minutes(1)
    private var accountUpdateTimer: DispatchSourceTimer?

    private let logger = Logger(label: "\(WelcomeInteractor.self)")
    private var tunnelObserver: TunnelObserver?
    private(set) var products: [SKProduct]?

    var didAddMoreCredit: (() -> Void)?

    var viewWillAppear = false {
        didSet {
            guard viewWillAppear else { return }
            startAccountUpdateTimer()
        }
    }

    var viewDidDisappear = false {
        didSet {
            guard viewDidDisappear else { return }
            stopAccountUpdateTimer()
        }
    }

    var accountNumber: String {
        tunnelManager.deviceState.accountData?.number ?? ""
    }

    var viewModel: WelcomeViewModel {
        WelcomeViewModel(
            deviceName: tunnelManager.deviceState.deviceData?.capitalizedName ?? "",
            accountNumber: tunnelManager.deviceState.accountData?.number.formattedAccountNumber ?? ""
        )
    }

    init(
        storePaymentManager: StorePaymentManager,
        tunnelManager: TunnelManager
    ) {
        self.storePaymentManager = storePaymentManager
        self.tunnelManager = tunnelManager
        let tunnelObserver =
            TunnelBlockObserver(didUpdateDeviceState: { [weak self] _, deviceState, previousDeviceState in
                let isInactive = previousDeviceState.accountData?.isExpired == true
                let isActive = deviceState.accountData?.isExpired == false
                if isInactive && isActive {
                    self?.didAddMoreCredit?()
                }
            })

        tunnelManager.addObserver(tunnelObserver)
        self.tunnelObserver = tunnelObserver
    }

    func requestProducts(
        with productIdentifiers: Set<StoreSubscription>,
        completionHandler: @Sendable @escaping (Result<SKProductsResponse, Error>) -> Void
    ) -> Cancellable {
        storePaymentManager.requestProducts(
            with: productIdentifiers,
            completionHandler: completionHandler
        )
    }

    private func startAccountUpdateTimer() {
        logger.debug(
            "Start polling account updates every \(accountUpdateTimerInterval) second(s)."
        )

        let timer = DispatchSource.makeTimerSource(queue: .main)
        timer.setEventHandler { [weak self] in
            self?.tunnelManager.updateAccountData()
        }

        accountUpdateTimer?.cancel()
        accountUpdateTimer = timer

        timer.schedule(
            wallDeadline: .now() + accountUpdateTimerInterval,
            repeating: accountUpdateTimerInterval.timeInterval
        )
        timer.activate()
    }

    private func stopAccountUpdateTimer() {
        logger.debug(
            "Stop polling account updates."
        )

        accountUpdateTimer?.cancel()
        accountUpdateTimer = nil
    }
}
