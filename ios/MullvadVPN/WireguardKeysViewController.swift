//
//  WireguardKeysViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 04/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit
import Logging

/// A UI refresh interval for the public key creation date (in seconds)
private let creationDateRefreshInterval = Int(60)

/// A maximum number of characters to display out of the entire public key representation
private let displayPublicKeyMaxLength = 20

private enum WireguardKeysViewState {
    case `default`
    case verifyingKey
    case verifiedKey(Bool)
    case regeneratingKey
    case regeneratedKey(Bool)
}

class WireguardKeysViewController: UIViewController, TunnelObserver {

    private let contentView: WireguardKeysContentView = {
        let contentView = WireguardKeysContentView()
        contentView.translatesAutoresizingMaskIntoConstraints = false
        return contentView
    }()

    private var publicKeyPeriodicUpdateTimer: DispatchSourceTimer?
    private var copyToPasteboardWork: DispatchWorkItem?
    private var updateDeviceTask: Cancellable?

    private let alertPresenter = AlertPresenter()
    private var state: WireguardKeysViewState = .default {
        didSet {
            updateViewState(state)
        }
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .secondaryColor

        let scrollView = UIScrollView()
        scrollView.translatesAutoresizingMaskIntoConstraints = false
        scrollView.addSubview(contentView)
        view.addSubview(scrollView)

        NSLayoutConstraint.activate([
            scrollView.topAnchor.constraint(equalTo: view.topAnchor),
            scrollView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            scrollView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            scrollView.bottomAnchor.constraint(equalTo: view.bottomAnchor),

            contentView.topAnchor.constraint(equalTo: scrollView.topAnchor),
            contentView.bottomAnchor.constraint(greaterThanOrEqualTo: scrollView.safeAreaLayoutGuide.bottomAnchor),
            contentView.leadingAnchor.constraint(equalTo: scrollView.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: scrollView.trailingAnchor),
            contentView.widthAnchor.constraint(equalTo: scrollView.widthAnchor),
        ])

        navigationItem.title = NSLocalizedString(
            "NAVIGATION_TITLE",
            tableName: "WireguardKeys",
            value: "WireGuard key",
            comment: ""
        )

        contentView.publicKeyRowView.actionHandler = { [weak self] in
            self?.copyPublicKey()
        }

        contentView.regenerateKeyButton.addTarget(self, action: #selector(handleRegenerateKey(_:)), for: .touchUpInside)
        contentView.verifyKeyButton.addTarget(self, action: #selector(handleVerifyKey(_:)), for: .touchUpInside)

        TunnelManager.shared.addObserver(self)
        updatePublicKey(deviceData: TunnelManager.shared.deviceState.deviceData, animated: false)

        startPublicKeyPeriodicUpdate()
    }

    private func startPublicKeyPeriodicUpdate() {
        let interval = DispatchTimeInterval.seconds(creationDateRefreshInterval)
        let timerSource = DispatchSource.makeTimerSource(queue: .main)
        timerSource.setEventHandler { [weak self] () -> Void in
            self?.updatePublicKey(deviceData: TunnelManager.shared.deviceState.deviceData, animated: true)
        }
        timerSource.schedule(deadline: .now() + interval, repeating: interval)
        timerSource.activate()

        self.publicKeyPeriodicUpdateTimer = timerSource
    }

    // MARK: - TunnelObserver

    func tunnelManagerDidLoadConfiguration(_ manager: TunnelManager) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelState tunnelState: TunnelState) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelSettings: TunnelSettingsV2) {
       // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateDeviceState deviceState: DeviceState) {
        updatePublicKey(deviceData: deviceState.deviceData, animated: true)
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: Error) {
        // no-op
    }

    // MARK: - Actions

    private func copyPublicKey() {
        guard let deviceData = TunnelManager.shared.deviceState.deviceData else { return }

        UIPasteboard.general.string = deviceData.wgKeyData.privateKey.publicKey.base64Key

        setPublicKeyTitle(
            string: NSLocalizedString(
                "COPIED_TO_PASTEBOARD_LABEL",
                tableName: "WireguardKeys",
                value: "COPIED TO PASTEBOARD!",
                comment: ""
            ),
            animated: true)

        let workItem = DispatchWorkItem { [weak self] in
            self?.updatePublicKey(
                deviceData: TunnelManager.shared.deviceState.deviceData,
                animated: true
            )
        }

        DispatchQueue.main.asyncAfter(wallDeadline: .now() + .seconds(3), execute: workItem)
        copyToPasteboardWork?.cancel()
        copyToPasteboardWork = workItem
    }

    @objc private func handleRegenerateKey(_ sender: Any) {
        regeneratePrivateKey()
    }

    @objc private func handleVerifyKey(_ sender: Any) {
        verifyKey()
    }

    // MARK: - Private

    private func formatKeyGenerationElapsedTime(with creationDate: Date) -> String? {
        return CustomDateComponentsFormatting.localizedString(
            from: creationDate,
            to: Date(),
            unitsStyle: .full
        ).map { (formattedInterval) -> String in
            return String(
                format: NSLocalizedString(
                    "KEY_GENERATED_SINCE_FORMAT",
                    tableName: "WireguardKeys",
                    value: "%@ ago",
                    comment: ""
                ),
                formattedInterval
            )
        }
    }

    private func updateCreationDateLabel(with creationDate: Date) {
        contentView.creationRowView.value = formatKeyGenerationElapsedTime(with: creationDate) ?? "-"
    }

    private func updatePublicKey(deviceData: StoredDeviceData?, animated: Bool) {
        if let wgKeyData = deviceData?.wgKeyData {
            let displayKey = wgKeyData.privateKey
                .publicKey
                .base64Key
                .prefix(displayPublicKeyMaxLength)
                .appending("...")

            setPublicKeyTitle(string: displayKey, animated: animated)
            updateCreationDateLabel(with: wgKeyData.creationDate)
        } else {
            setPublicKeyTitle(string: "-", animated: animated)
            contentView.creationRowView.value = "-"
        }
    }

    private func updateViewState(_ state: WireguardKeysViewState) {
        switch state {
        case .default:
            setKeyActionButtonsEnabled(true)
            contentView.publicKeyRowView.status = .default

        case .verifyingKey:
            setKeyActionButtonsEnabled(false)
            contentView.publicKeyRowView.status = .verifying

        case .verifiedKey(let isValid):
            setKeyActionButtonsEnabled(true)
            contentView.publicKeyRowView.status = .verified(isValid)
            announceKeyVerificationResult(isValid: isValid)

        case .regeneratingKey:
            setKeyActionButtonsEnabled(false)
            contentView.publicKeyRowView.status = .regenerating

        case .regeneratedKey(let success):
            setKeyActionButtonsEnabled(true)
            contentView.publicKeyRowView.status = .default
            if success {
                announceKeyRegenerated()
            }

        }
    }

    private func setKeyActionButtonsEnabled(_ enabled: Bool) {
        contentView.regenerateKeyButton.isEnabled = enabled
        contentView.verifyKeyButton.isEnabled = enabled
    }

    private func verifyKey() {
        updateViewState(.verifyingKey)

        updateDeviceTask?.cancel()

        updateDeviceTask = TunnelManager.shared.updateDeviceData { [weak self] completion in
            guard let self = self else { return }

            switch completion {
            case .success:
                self.updateViewState(.verifiedKey(true))

            case .failure(let error):
                if error is RevokedDeviceError {
                    self.updateViewState(.verifiedKey(false))
                } else {
                    self.showKeyVerificationFailureAlert(error)
                    self.updateViewState(.default)
                }

            case .cancelled:
                break
            }
        }
    }

    private func regeneratePrivateKey() {
        self.updateViewState(.regeneratingKey)

        _ = TunnelManager.shared.rotatePrivateKey(forceRotate: true) { [weak self] completion in
            if let error = completion.error {
                self?.showKeyRegenerationFailureAlert(error)
                self?.updateViewState(.regeneratedKey(false))
            } else {
                self?.updateViewState(.regeneratedKey(true))
            }
        }
    }

    private func showKeyVerificationFailureAlert(_ error: Error) {
        let errorDescription = String(
            format: NSLocalizedString(
                "VERIFY_KEY_FAILURE_ALERT_MESSAGE",
                tableName: "WireguardKeys",
                value: "Failed to verify the WireGuard key: %@",
                comment: ""
            ),
            error.localizedDescription
        )

        let alertController = UIAlertController(
            title: NSLocalizedString(
                "VERIFY_KEY_FAILURE_ALERT_TITLE",
                tableName: "WireguardKeys",
                value: "Cannot verify the key",
                comment: ""
            ),
            message: errorDescription,
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "VERIFY_KEY_FAILURE_ALERT_OK_ACTION",
                    tableName: "WireguardKeys",
                    value: "OK",
                    comment: ""
                ),
                style: .cancel
            )
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func showKeyRegenerationFailureAlert(_ error: Error) {
        let alertController = UIAlertController(
            title: NSLocalizedString(
                "REGENERATE_KEY_FAILURE_ALERT_TITLE",
                tableName: "WireguardKeys",
                value: "Cannot regenerate the key",
                comment: ""
            ),
            message: error.localizedDescription,
            preferredStyle: .alert
        )
        alertController.addAction(
            UIAlertAction(
                title: NSLocalizedString(
                    "REGENERATE_KEY_FAILURE_ALERT_OK_ACTION",
                    tableName: "WireguardKeys",
                    value: "OK",
                    comment: ""
                ),
                style: .cancel
            )
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }


    private func setPublicKeyTitle(string: String, animated: Bool) {
        let updateTitle = {
            self.contentView.publicKeyRowView.value = string
        }

        if animated {
            updateTitle()
        } else {
            UIView.performWithoutAnimation {
                updateTitle()
                self.contentView.publicKeyRowView.layoutIfNeeded()
            }
        }
    }

    private func announceKeyVerificationResult(isValid: Bool) {
        let announcementString: String

        if isValid {
            announcementString = NSLocalizedString(
                "ACCESSIBILITY_ANNOUNCEMENT_VALID_KEY",
                tableName: "WireguardKeys",
                value: "Key is valid.",
                comment: ""
            )
        } else {
            announcementString = NSLocalizedString(
                "ACCESSIBILITY_ANNOUNCEMENT_INVALID_KEY",
                tableName: "WireguardKeys",
                value: "Key is invalid.",
                comment: ""
            )
        }

        UIAccessibility.post(notification: .announcement, argument: announcementString)
    }

    private func announceKeyRegenerated() {
        let announcementString = NSLocalizedString(
            "ACCESSIBILITY_ANNOUNCEMENT_REGENERATED_KEY",
            tableName: "WireguardKeys",
            value: "Key is regenerated.",
            comment: ""
        )
        UIAccessibility.post(notification: .announcement, argument: announcementString)
    }

}
