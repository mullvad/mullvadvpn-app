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
}

class WireguardKeysViewController: UIViewController, TunnelObserver {

    @IBOutlet var publicKeyButton: UIButton!
    @IBOutlet var creationDateLabel: UILabel!
    @IBOutlet var regenerateKeyButton: UIButton!
    @IBOutlet var verifyKeyButton: UIButton!
    @IBOutlet var wireguardKeyStatusView: WireguardKeyStatusView!

    private var publicKeyPeriodicUpdateTimer: DispatchSourceTimer?
    private var copyToPasteboardWork: DispatchWorkItem?

    private let alertPresenter = AlertPresenter()
    private let logger = Logger(label: "WireguardKeysViewController")

    private var state: WireguardKeysViewState = .default {
        didSet {
            updateViewState(state)
        }
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        navigationItem.title = NSLocalizedString("WireGuard key", comment: "Navigation title")

        TunnelManager.shared.addObserver(self)
        updatePublicKeyWithMetadata(publicKeyWithMetadata: TunnelManager.shared.publicKeyWithMetadata, animated: false)

        startPublicKeyPeriodicUpdate()
    }

    private func startPublicKeyPeriodicUpdate() {
        let interval = DispatchTimeInterval.seconds(kCreationDateRefreshInterval)
        let timerSource = DispatchSource.makeTimerSource(queue: .main)
        timerSource.setEventHandler { [weak self] () -> Void in
            let metadata = TunnelManager.shared.publicKeyWithMetadata

            self?.updatePublicKeyWithMetadata(publicKeyWithMetadata: metadata, animated: true)
        }
        timerSource.schedule(deadline: .now() + interval, repeating: interval)
        timerSource.activate()

        self.publicKeyPeriodicUpdateTimer = timerSource
    }

    // MARK: - TunnelObserver

    func tunnelStateDidChange(tunnelState: TunnelState) {
        // no-op
    }

    func tunnelPublicKeyDidChange(publicKeyWithMetadata: PublicKeyWithMetadata?) {
        DispatchQueue.main.async {
            self.updatePublicKeyWithMetadata(publicKeyWithMetadata: publicKeyWithMetadata, animated: true)
        }
    }

    // MARK: - IBActions

    @IBAction func copyPublicKey(_ sender: Any) {
        guard let metadata = TunnelManager.shared.publicKeyWithMetadata else { return }

        UIPasteboard.general.string = metadata.stringRepresentation()

        setPublicKeyTitle(
            string: NSLocalizedString("COPIED TO PASTEBOARD!", comment: ""),
            animated: true)

        let dispatchWork = DispatchWorkItem { [weak self] in
            let metadata = TunnelManager.shared.publicKeyWithMetadata

            self?.updatePublicKeyWithMetadata(publicKeyWithMetadata: metadata, animated: true)
        }

        DispatchQueue.main.asyncAfter(wallDeadline: .now() + .seconds(3), execute: dispatchWork)

        self.copyToPasteboardWork?.cancel()
        self.copyToPasteboardWork = dispatchWork
    }

    @IBAction func handleRegenerateKey(_ sender: Any) {
        regeneratePrivateKey()
    }

    @IBAction func handleVerifyKey(_ sender: Any) {
        verifyKey()
    }

    // MARK: - Private

    private func formatKeyGenerationElapsedTime(with creationDate: Date) -> String? {
        return CustomDateComponentsFormatting.localizedString(
            from: creationDate,
            to: Date(),
            unitsStyle: .full
        ).map { (formattedInterval) -> String in
            return String(format: NSLocalizedString("%@ ago", comment: ""), formattedInterval)
        }
    }

    private func updateCreationDateLabel(with creationDate: Date) {
        creationDateLabel.text = formatKeyGenerationElapsedTime(with: creationDate) ?? "-"
    }

    private func updatePublicKeyWithMetadata(publicKeyWithMetadata: PublicKeyWithMetadata?, animated: Bool) {
        if let publicKey = publicKeyWithMetadata {
            let displayKey = publicKey
                .stringRepresentation(maxLength: kDisplayPublicKeyMaxLength)

            setPublicKeyTitle(string: displayKey, animated: animated)
            updateCreationDateLabel(with: publicKey.creationDate)
        } else {
            setPublicKeyTitle(string: "-", animated: animated)
            creationDateLabel.text = "-"
        }
    }

    private func updateViewState(_ state: WireguardKeysViewState) {
        switch state {
        case .default:
            setKeyActionButtonsEnabled(true)
            wireguardKeyStatusView.status = .default

        case .verifyingKey:
            setKeyActionButtonsEnabled(false)
            wireguardKeyStatusView.status = .verifying

        case .verifiedKey(let isValid):
            setKeyActionButtonsEnabled(true)
            wireguardKeyStatusView.status = .verified(isValid)

        case .regeneratingKey:
            setKeyActionButtonsEnabled(false)
            wireguardKeyStatusView.status = .verifying
        }
    }

    private func setKeyActionButtonsEnabled(_ enabled: Bool) {
        regenerateKeyButton.isEnabled = enabled
        verifyKeyButton.isEnabled = enabled
    }

    private func verifyKey() {
        self.updateViewState(.verifyingKey)

        TunnelManager.shared.verifyPublicKey { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success(let isValid):
                    self.updateViewState(.verifiedKey(isValid))

                case .failure(let error):
                    let alertController = UIAlertController(
                        title: NSLocalizedString("Cannot verify the key", comment: ""),
                        message: error.errorChainDescription,
                        preferredStyle: .alert
                    )
                    alertController.addAction(
                        UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
                    )

                    self.alertPresenter.enqueue(alertController, presentingController: self)
                    self.updateViewState(.default)
                }
            }
        }
    }

    private func regeneratePrivateKey() {
        self.updateViewState(.regeneratingKey)

        TunnelManager.shared.regeneratePrivateKey { (result) in
            DispatchQueue.main.async {
                switch result {
                case .success:
                    break

                case .failure(let error):
                    let alertController = UIAlertController(
                        title: NSLocalizedString("Cannot regenerate the key", comment: ""),
                        message: error.errorChainDescription,
                        preferredStyle: .alert
                    )
                    alertController.addAction(
                        UIAlertAction(title: NSLocalizedString("OK", comment: ""), style: .cancel)
                    )

                    self.logger.error(chainedError: error, message: "Failed to regenerate the private key")

                    self.alertPresenter.enqueue(alertController, presentingController: self)
                }

                self.updateViewState(.default)
            }
        }
    }

    private func setPublicKeyTitle(string: String, animated: Bool) {
        let updateTitle = {
            self.publicKeyButton.setTitle(string, for: .normal)
        }

        if animated {
            updateTitle()
        } else {
            UIView.performWithoutAnimation {
                updateTitle()
                publicKeyButton.layoutIfNeeded()
            }
        }
    }

}

class WireguardKeyStatusView: UIView {

    enum Status {
        case `default`, verifying, verified(Bool)
    }

    @IBOutlet var textLabel: UILabel!
    @IBOutlet var activityIndicator: SpinnerActivityIndicatorView!

    var status: Status = .default {
        didSet {
            updateView()
        }
    }

    override func awakeFromNib() {
        super.awakeFromNib()

        updateView()
    }

    private func updateView() {
        switch status {
        case .default:
            textLabel.isHidden = true
            activityIndicator.stopAnimating()

        case .verifying:
            textLabel.isHidden = true
            activityIndicator.startAnimating()

        case .verified(let isValid):
            textLabel.isHidden = false
            activityIndicator.stopAnimating()

            if isValid {
                textLabel.textColor = .successColor
                textLabel.text = NSLocalizedString("Key is valid", comment: "")
            } else {
                textLabel.textColor = .dangerColor
                textLabel.text = NSLocalizedString("Key is invalid", comment: "")
            }
        }
    }

}
