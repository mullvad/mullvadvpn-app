//
//  RedeemVoucherViewConrtoller.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import Foundation
import UIKit

class RedeemVoucherViewController: UIViewController {
    enum RedeemVoucherState {
        case initial, waiting, success, failure
    }

    private var redeemVoucherState = RedeemVoucherState.initial

    private let apiProxy = REST.ProxyFactory.shared.createAPIProxy()

    private lazy var contentView = RedeemVoucherContentView()

    private var isVoucherLengthSatisfied = false {
        didSet {
            if isVoucherLengthSatisfied != oldValue {
                updateViewState(animated: true)
            }
        }
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    override func viewDidLoad() {
        setUpContentView()
        setUpButtonTargets()
        addTextFieldObserver()
        updateViewState(animated: false)
    }
}

// MARK: - Private Functions

private extension RedeemVoucherViewController {
    func setUpContentView() {
        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    func setUpButtonTargets() {
        contentView.redeemButton.addTarget(
            self,
            action: #selector(didTapRedeemButton),
            for: .touchUpInside
        )

        contentView.cancelButton.addTarget(
            self,
            action: #selector(didTapCancelButton),
            for: .touchUpInside
        )
    }

    func addTextFieldObserver() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextField.textDidChangeNotification,
            object: contentView.inputTextField
        )
    }

    @objc func textDidChange() {
        updateIsVoucherLengthSatisfied()
        if isVoucherLengthSatisfied {
            dismissKeyboard()
        }
    }

    func updateIsVoucherLengthSatisfied() {
        isVoucherLengthSatisfied = contentView.inputTextField.text?.count == contentView
            .inputTextField.placeholder?.count
    }

    func dismissKeyboard() {
        contentView.inputTextField.resignFirstResponder()
    }

    @objc func didTapRedeemButton() {
        submitVoucher()
    }

    @objc func didTapCancelButton() {
        rootContainerController?.popViewController(animated: true)
    }

    func setRedeemVoucherState(_ state: RedeemVoucherState, animated: Bool) {
        redeemVoucherState = state
        updateViewState(animated: true)
    }

    func updateViewState(animated: Bool) {
        let isInteractionEnabled = redeemVoucherState != .waiting

        let actions = { [weak self] in
            guard let self = self else { return }

            self.setActivityInditatorIsAnimating(!isInteractionEnabled)
            self.contentView.activityIndicator.alpha = isInteractionEnabled ? 0 : 1
            self.contentView.redeemButton.isEnabled = isInteractionEnabled && self
                .isVoucherLengthSatisfied && self.redeemVoucherState != .success
            self.contentView.cancelButton.isEnabled = self.redeemVoucherState != .success
            self.contentView.statusLabel.alpha = self.redeemVoucherState == .initial || self
                .redeemVoucherState == .success
                ? 0
                : 1
            self.contentView.statusLabel.text = self.statusLabelText(for: self.redeemVoucherState)
            self.contentView.statusLabel.textColor = self.redeemVoucherState == .failure
                ? .dangerColor
                : .white
        }

        if animated {
            UIView.animate(withDuration: 0.25) { [weak self] in
                guard let self = self else { return }

                actions()
                self.view.layoutIfNeeded()
            }
        } else {
            actions()
        }
    }

    func statusLabelText(for state: RedeemVoucherState) -> String {
        switch state {
        case .failure:
            return NSLocalizedString(
                "REDEEM_VOUCHER_STATUS_FAILURE",
                tableName: "RedeemVoucher",
                value: "Voucher code is invalid.",
                comment: ""
            )
        default:
            return NSLocalizedString(
                "REDEEM_VOUCHER_STATUS_WAITING",
                tableName: "RedeemVoucher",
                value: "Verifying voucher...",
                comment: ""
            )
        }
    }

    func setActivityInditatorIsAnimating(_ isAnimating: Bool) {
        if isAnimating {
            contentView.activityIndicator.startAnimating()
        } else {
            contentView.activityIndicator.stopAnimating()
        }
    }
}

// MARK: - API

private extension RedeemVoucherViewController {
    private func submitVoucher() {
        guard let voucherCode = contentView.inputTextField.text,
              let accountNumber = TunnelManager.shared.deviceState.accountData?.number
        else { return }

        setRedeemVoucherState(.waiting, animated: true)

        let request = REST.SubmitVoucherRequest(voucherCode: voucherCode)

        let group = DispatchGroup()
        group.enter()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.5) {
            group.leave()
        }

        apiProxy.submitVoucher(
            request,
            accountNumber: accountNumber,
            retryStrategy: .default
        ) { completion in
            group.notify(queue: .main) { [weak self] in
                guard let self = self else { return }
                switch completion {
                case let .success(submitVoucherResponse):
                    self.setRedeemVoucherState(.success, animated: true)
                    self.rootContainerController?.pushViewController(
                        RedeemVoucherSuccessViewController(
                            timeAdded: submitVoucherResponse.timeAdded,
                            newExpiry: submitVoucherResponse.newExpiry
                        ),
                        animated: true
                    )
                case .failure:
                    self.setRedeemVoucherState(.failure, animated: true)
                default:
                    break
                }
            }
        }
    }
}
