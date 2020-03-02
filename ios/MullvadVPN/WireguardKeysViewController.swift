//
//  WireguardKeysViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 04/12/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import Foundation
import UIKit
import os

/// A UI refresh interval for the public key creation date (in seconds)
private let kCreationDateRefreshInterval = TimeInterval(60)

/// A maximum number of characters to display out of the entire public key representation
private let kDisplayPublicKeyMaxLength = 20

private enum WireguardKeysViewState {
    case `default`
    case verifyingKey
    case verifiedKey(Bool)
    case regeneratingKey
}

enum VerifyWireguardPublicKeyError {
    case network(MullvadAPI.Error)
    case server(MullvadAPI.ResponseError)
}

extension VerifyWireguardPublicKeyError: LocalizedError {
    var errorDescription: String? {
        return NSLocalizedString("Cannot verify the public key", comment: "")
    }

    var failureReason: String? {
        switch self {
        case .network(.network(let urlError)):
            return urlError.localizedDescription
        case .server(let serverError):
            return serverError.errorDescription
        default:
            return NSLocalizedString("Internal error", comment: "")
        }
    }
}

class WireguardKeysViewController: UIViewController {

    @IBOutlet var publicKeyButton: UIButton!
    @IBOutlet var creationDateLabel: UILabel!
    @IBOutlet var regenerateKeyButton: UIButton!
    @IBOutlet var verifyKeyButton: UIButton!
    @IBOutlet var wireguardKeyStatusView: WireguardKeyStatusView!

    private var fetchKeySubscriber: AnyCancellable?
    private var verifyKeySubscriber: AnyCancellable?
    private var regenerateKeySubscriber: AnyCancellable?
    private var creationDateTimerSubscriber: AnyCancellable?
    private var copyToPasteboardSubscriber: AnyCancellable?

    private let apiClient = MullvadAPI()
    private var publicKey: WireguardPublicKey?

    private var state: WireguardKeysViewState = .default {
        didSet {
            updateViewState(state)
        }
    }

    private lazy var relativeFormatter: DateComponentsFormatter = {
        let formatter = DateComponentsFormatter()
        formatter.unitsStyle = .full
        formatter.allowedUnits = [.minute, .hour, .day, .month, .year]
        formatter.maximumUnitCount = 1

        return formatter
    }()

    override func viewDidLoad() {
        super.viewDidLoad()

        // Reset Storyboard placeholders
        setPublicKeyTitle(string: "-", animated: false)
        creationDateLabel.text = "-"

        creationDateTimerSubscriber = Timer.publish(every: kCreationDateRefreshInterval, on: .main, in: .common)
            .autoconnect()
            .sink { [weak self] _ in
                guard let self = self else { return }

                if let creationDate = self.publicKey?.creationDate {
                    self.updateCreationDateLabel(with: creationDate)
                }
        }

        loadPublicKey(animated: false)
    }

    // MARK: - IBActions

    @IBAction func copyPublicKey(_ sender: Any) {
        guard let publicKey = self.publicKey else { return }

        UIPasteboard.general.string = publicKey.stringRepresentation()

        setPublicKeyTitle(
            string: NSLocalizedString("COPIED TO PASTEBOARD!", comment: ""),
            animated: true)

        copyToPasteboardSubscriber =
            Just(()).delay(for: .seconds(3), scheduler: DispatchQueue.main)
                .sink(receiveValue: { [weak self] _ in
                    guard let self = self, let publicKey = self.publicKey else { return }

                    let displayKey = publicKey
                        .stringRepresentation(maxLength: kDisplayPublicKeyMaxLength)

                    self.setPublicKeyTitle(string: displayKey, animated: true)
                })
    }

    @IBAction func handleRegenerateKey(_ sender: Any) {
        regeneratePrivateKey()
    }

    @IBAction func handleVerifyKey(_ sender: Any) {
        guard let accountToken = Account.shared.token,
            let publicKey = publicKey else { return }

        verifyKey(accountToken: accountToken, publicKey: publicKey)
    }

    // MARK: - Private

    private func formatKeyGenerationElapsedTime(with creationDate: Date) -> String? {
        let elapsedTime = Date().timeIntervalSince(creationDate)

        if elapsedTime >= 60 {
            if let formattedInterval = relativeFormatter.string(from: elapsedTime) {
                return String.localizedStringWithFormat(
                    NSLocalizedString("%@ ago", comment: ""),
                    formattedInterval)
            } else {
                return nil
            }
        } else {
            return NSLocalizedString("Less than a minute ago", comment: "")
        }
    }

    private func updateCreationDateLabel(with creationDate: Date) {
        creationDateLabel.text = formatKeyGenerationElapsedTime(with: creationDate) ?? "-"
    }

    private func loadPublicKey(animated: Bool) {
        fetchKeySubscriber = TunnelManager.shared.getWireguardPublicKey()
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { (completion) in
                switch completion {
                case .finished:
                    break

                case .failure(let error):
                    os_log(.error, "Failed to receive the public key for Wireguard: %{public}s",
                           error.localizedDescription)

                    self.presentError(error, preferredStyle: .alert)
                }
            }) { [weak self] (publicKey) in
                guard let self = self else { return }

                let displayKey = publicKey
                    .stringRepresentation(maxLength: kDisplayPublicKeyMaxLength)

                self.setPublicKeyTitle(string: displayKey, animated: animated)
                self.updateCreationDateLabel(with: publicKey.creationDate)

                self.publicKey = publicKey
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

    private func verifyKey(accountToken: String, publicKey: WireguardPublicKey) {
        verifyKeySubscriber = apiClient.checkWireguardKey(
            accountToken: accountToken,
            publicKey: publicKey.rawRepresentation
        )
            .retry(1)
            .receive(on: DispatchQueue.main)
            .mapError { VerifyWireguardPublicKeyError.network($0) }
            .flatMap({ (response) in
                response.result
                    .mapError { VerifyWireguardPublicKeyError.server($0) }
                    .publisher
            })
            .handleEvents(receiveSubscription: { _ in
                self.updateViewState(.verifyingKey)
            })
            .sink(receiveCompletion: { (completion) in
                switch completion {
                case .finished:
                    break

                case .failure(let error):
                    self.presentError(error, preferredStyle: .alert)
                    self.updateViewState(.default)
                }
            }) { (isValid) in
                self.updateViewState(.verifiedKey(isValid))
        }
    }

    private func regeneratePrivateKey() {
        regenerateKeySubscriber = TunnelManager.shared.regeneratePrivateKey()
            .receive(on: DispatchQueue.main)
            .handleEvents(receiveSubscription: { (_) in
                self.updateViewState(.regeneratingKey)
            }, receiveCompletion: { (completion) in
                self.updateViewState(.default)
            })
            .sink { (completion) in
                switch completion {
                case .finished:
                    self.loadPublicKey(animated: true)

                case .failure(let error):
                    os_log(.error, "Failed to re-generate the private key: %{public}s",
                           error.errorDescription ?? "")

                    self.presentError(error, preferredStyle: .alert)
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
