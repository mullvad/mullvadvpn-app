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
private let kCreationDateRefreshInterval = Int(60)

/// A maximum number of characters to display out of the entire public key representation
private let kDisplayPublicKeyMaxLength = 20

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
    private var verifyKeyCancellable: Cancellable?

    private let alertPresenter = AlertPresenter()
    private var state: WireguardKeysViewState = .default {
        didSet {
            updateViewState(state)
        }
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    private let apiProxy = REST.ProxyFactory.shared.createAPIProxy()

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

        navigationItem.title = NSLocalizedString("NAVIGATION_TITLE", tableName: "WireguardKeys", comment: "")

        contentView.publicKeyRowView.actionHandler = { [weak self] in
            self?.copyPublicKey()
        }

        contentView.regenerateKeyButton.addTarget(self, action: #selector(handleRegenerateKey(_:)), for: .touchUpInside)
        contentView.verifyKeyButton.addTarget(self, action: #selector(handleVerifyKey(_:)), for: .touchUpInside)

        TunnelManager.shared.addObserver(self)
        updatePublicKey(tunnelSettings: TunnelManager.shared.tunnelInfo?.tunnelSettings, animated: false)

        startPublicKeyPeriodicUpdate()
    }

    private func startPublicKeyPeriodicUpdate() {
        let interval = DispatchTimeInterval.seconds(kCreationDateRefreshInterval)
        let timerSource = DispatchSource.makeTimerSource(queue: .main)
        timerSource.setEventHandler { [weak self] () -> Void in
            self?.updatePublicKey(tunnelSettings: TunnelManager.shared.tunnelInfo?.tunnelSettings, animated: true)
        }
        timerSource.schedule(deadline: .now() + interval, repeating: interval)
        timerSource.activate()

        self.publicKeyPeriodicUpdateTimer = timerSource
    }

    // MARK: - TunnelObserver

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelState tunnelState: TunnelState) {
        // no-op
    }

    func tunnelManager(_ manager: TunnelManager, didUpdateTunnelSettings tunnelInfo: TunnelInfo?) {
        self.updatePublicKey(tunnelSettings: tunnelInfo?.tunnelSettings, animated: true)
    }

    func tunnelManager(_ manager: TunnelManager, didFailWithError error: TunnelManager.Error) {
        // no-op
    }

    // MARK: - Actions

    private func copyPublicKey() {
        guard let tunnelInfo = TunnelManager.shared.tunnelInfo else { return }

        let metadata = tunnelInfo.tunnelSettings.interface.privateKey.publicKeyWithMetadata

        UIPasteboard.general.string = metadata.stringRepresentation()

        setPublicKeyTitle(
            string: NSLocalizedString("COPIED_TO_PASTEBOARD_LABEL", tableName: "WireguardKeys", comment: ""),
            animated: true)

        let workItem = DispatchWorkItem { [weak self] in
            let tunnelSettings = TunnelManager.shared.tunnelInfo?.tunnelSettings

            self?.updatePublicKey(tunnelSettings: tunnelSettings, animated: true)
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
            return String(format: NSLocalizedString("KEY_GENERATED_SINCE_FORMAT", tableName: "WireguardKeys", comment: ""), formattedInterval)
        }
    }

    private func updateCreationDateLabel(with creationDate: Date) {
        contentView.creationRowView.value = formatKeyGenerationElapsedTime(with: creationDate) ?? "-"
    }

    private func updatePublicKey(tunnelSettings: TunnelSettings?, animated: Bool) {
        if let publicKey = tunnelSettings?.interface.privateKey.publicKeyWithMetadata {
            let displayKey = publicKey
                .stringRepresentation(maxLength: kDisplayPublicKeyMaxLength)

            setPublicKeyTitle(string: displayKey, animated: animated)
            updateCreationDateLabel(with: publicKey.creationDate)
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
        guard let tunnelInfo = TunnelManager.shared.tunnelInfo else { return }

        updateViewState(.verifyingKey)

        verifyKeyCancellable?.cancel()

        verifyKeyCancellable = apiProxy.getWireguardKey(
            accountNumber: tunnelInfo.token,
            publicKey: tunnelInfo.tunnelSettings.interface.publicKey,
            retryStrategy: .default
        ) { [weak self] result in
            guard let self = self else { return }

            switch result {
            case .success:
                self.updateViewState(.verifiedKey(true))

            case .failure(let error):
                if case .unhandledResponse(_, let serverErrorResponse) = error,
                   serverErrorResponse?.code == .publicKeyNotFound {
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

        TunnelManager.shared.regeneratePrivateKey { [weak self] error in
            if let error = error {
                self?.showKeyRegenerationFailureAlert(error)
                self?.updateViewState(.regeneratedKey(false))
            } else {
                self?.updateViewState(.regeneratedKey(true))
            }
        }
    }

    private func showKeyVerificationFailureAlert(_ error: REST.Error) {
        let reason = error.errorChainDescription ?? ""
        let errorDescription = String(
            format: NSLocalizedString(
                "VERIFY_KEY_FAILURE_ALERT_MESSAGE",
                tableName: "WireguardKeys",
                value: "Failed to verify the WireGuard key on server: %@",
                comment: ""
            ),
            reason
        )

        let alertController = UIAlertController(
            title: NSLocalizedString("VERIFY_KEY_FAILURE_ALERT_TITLE", tableName: "WireguardKeys", comment: ""),
            message: errorDescription,
            preferredStyle: .alert
        )

        alertController.addAction(
            UIAlertAction(title: NSLocalizedString("VERIFY_KEY_FAILURE_ALERT_OK_ACTION", tableName: "WireguardKeys", comment: ""), style: .cancel)
        )

        alertPresenter.enqueue(alertController, presentingController: self)
    }

    private func showKeyRegenerationFailureAlert(_ error: TunnelManager.Error) {
        let alertController = UIAlertController(
            title: NSLocalizedString("REGENERATE_KEY_FAILURE_ALERT_TITLE", tableName: "WireguardKeys", comment: ""),
            message: error.errorChainDescription,
            preferredStyle: .alert
        )
        alertController.addAction(
            UIAlertAction(title: NSLocalizedString("REGENERATE_KEY_FAILURE_ALERT_OK_ACTION", tableName: "WireguardKeys", comment: ""), style: .cancel)
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
