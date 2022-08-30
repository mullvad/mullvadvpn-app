//
//  RedeemVoucherViewController.swift
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

    private let apiProxy = REST.ProxyFactory.shared.createAPIProxy()
    private let contentView = RedeemVoucherContentView()

    private let instructionLabelSuccessString = NSLocalizedString(
        "REDEEM_VOUCHER_INSTRUCTION_SUCCESS",
        tableName: "RedeemVoucher",
        value: "Voucher was successfully redeemed.",
        comment: ""
    )

    private let gotItButtonTitle = NSLocalizedString(
        "REDEEM_VOUCHER_GOT_IT_BUTTON",
        tableName: "RedeemVoucher",
        value: "Got it!",
        comment: ""
    )

    private var redeemVoucherState = RedeemVoucherState.initial
    private var didDismissOnSuccess: (() -> Void)?
    private var didAddTime: (() -> Void)?
    private var navigationControllerOriginY: CGFloat?
    private var isViewMoved = false

    private var timeAdded = "" {
        didSet { (didAddTime ?? {})() }
    }

    private var isVoucherLengthSatisfied = false {
        didSet {
            if isVoucherLengthSatisfied != oldValue {
                updateViewState(animated: true)
            }
        }
    }

    override var preferredStatusBarStyle: UIStatusBarStyle { .lightContent }

    init(
        didDismissOnSuccess: (() -> Void)? = nil,
        didAddTime: (() -> Void)? = nil
    ) {
        self.didDismissOnSuccess = didDismissOnSuccess
        self.didAddTime = didAddTime
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        setUpContentView()
        setUpButtonTargets()
        addObservers()
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

    @objc func didTapRedeemButton() {
        submitVoucher()
    }

    @objc func didTapCancelButton() {
        dismiss(animated: true)
    }

    func addObservers() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextField.textDidChangeNotification,
            object: contentView.inputTextField
        )

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillShow),
            name: UIResponder.keyboardWillShowNotification,
            object: nil
        )

        NotificationCenter.default.addObserver(
            self,
            selector: #selector(keyboardWillHide),
            name: UIResponder.keyboardWillHideNotification,
            object: nil
        )
    }

    @objc func textDidChange() {
        updateIsVoucherLengthSatisfied()
        if isVoucherLengthSatisfied {
            dismissKeyboard()
        }
    }

    func updateIsVoucherLengthSatisfied() {
        isVoucherLengthSatisfied =
            contentView.inputTextField.text?.count == contentView.inputTextField.placeholder?.count
    }

    func dismissKeyboard() {
        contentView.inputTextField.resignFirstResponder()
    }

    func setRedeemVoucherState(_ state: RedeemVoucherState, animated: Bool) {
        redeemVoucherState = state
        updateViewState(animated: true)
    }

    func updateViewState(animated: Bool) {
        UIView.performWithoutAnimation {
            self.animationSetup(for: self.redeemVoucherState)()
            self.view.layoutIfNeeded()
        }

        let actions = { [weak self] in
            guard let self = self else { return }

            self.setActivityInditatorIsAnimating(self.redeemVoucherState == .waiting)
            self.contentView.activityIndicator.alpha = self.redeemVoucherState == .waiting ? 1 : 0
            self.contentView.redeemButton.isEnabled = self.redeemVoucherState != .waiting
                && self.isVoucherLengthSatisfied
            self.contentView.statusLabel.alpha = self.redeemVoucherState == .initial ? 0 : 1
            self.contentView.statusLabel.text = self.statusLabelText(for: self.redeemVoucherState)
            self.contentView.statusLabel.textColor = self.redeemVoucherState == .failure
                ? .dangerColor
                : .white

            if self.redeemVoucherState == .success {
                self.contentView.inputTextField.constraints.height?.constant = 0
                self.contentView.inputTextField.alpha = 0

                self.contentView.redeemButton.constraints.height?.constant = 0
                self.contentView.redeemButton.alpha = 0

                self.contentView.instructionLabel.alpha = 1
                self.contentView.instructionLabel.text = self.instructionLabelSuccessString
                self.contentView.instructionLabel.font = UIFont.boldSystemFont(ofSize: 20)

                self.contentView.topStackView.spacing = UIMetrics.StackSpacing.close.rawValue / 2
                self.contentView.topStackTopConstraint.constant = UIMetrics.sectionSpacing

                self.contentView.successImageHeightConstraint.constant
                    = SpinnerActivityIndicatorView.Style.large.intrinsicSize.height
                self.contentView.successImage.alpha = 1

                self.contentView.statusLabel.alpha = 0.6

                self.contentView.cancelButton.setTitle(self.gotItButtonTitle, for: .normal)
            }
        }

        if animated {
            UIView.animate(
                withDuration: AnimationDuration.medium.rawValue,
                delay: 0,
                options: .curveEaseInOut
            ) {
                actions()
                self.view.layoutIfNeeded()
            } completion: { _ in
                self.onSuccessCompletion()
            }
        } else {
            actions()
        }
    }

    func animationSetup(for redeemVoucherState: RedeemVoucherState) -> (() -> Void) {
        switch redeemVoucherState {
        case .waiting:
            return {
                self.contentView.activityIndicator.alpha = 0
                self.contentView.activityIndicator.startAnimating()
            }
        case .failure:
            return {
                self.contentView.activityIndicator.stopAnimating()
                self.contentView.statusLabel.alpha = 0
            }
        case .success:
            return {
                self.contentView.instructionLabel.alpha = 0
            }
        default:
            return {}
        }
    }

    func onSuccessCompletion() {
        if redeemVoucherState == .success {
            contentView.cancelButton.addTarget(
                self,
                action: #selector(didTapGotIt),
                for: .touchUpInside
            )

            contentView.topStackView.spacing = UIMetrics.StackSpacing.close.rawValue
            contentView.inputTextField.removeFromSuperview()
            contentView.redeemButton.removeFromSuperview()
        }
    }

    @objc func didTapGotIt() {
        (didDismissOnSuccess ?? {})()
        dismiss(animated: true)
    }

    func statusLabelText(for state: RedeemVoucherState) -> String {
        switch state {
        case .success:
            return NSLocalizedString(
                "REDEEM_VOUCHER_STATUS_SUCCESS",
                tableName: "RedeemVoucher",
                value: "\(timeAdded) were added to your account.",
                comment: ""
            )
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

    private func submitVoucher() {
        guard let voucherCode = contentView.inputTextField.text,
              let accountNumber = TunnelManager.shared.deviceState.accountData?.number
        else { return }

        setRedeemVoucherState(.waiting, animated: true)

        let request = REST.SubmitVoucherRequest(voucherCode: voucherCode)

        let group = DispatchGroup()
        group.enter()
        DispatchQueue.main.asyncAfter(
            deadline: .now() + AnimationDuration.medium.rawValue * 2
        ) {
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
                    self.timeAdded = self.formattedTimeAdded(from: submitVoucherResponse.timeAdded)
                    self.setRedeemVoucherState(.success, animated: true)
                case .failure:
                    self.setRedeemVoucherState(.failure, animated: true)
                default:
                    break
                }
            }
        }
    }

    func formattedTimeAdded(from timeAdded: Int) -> String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full
        formatter.calendar?.locale = .usEnglish

        return formatter.string(from: Double(timeAdded)) ?? ""
    }
}

// MARK: - Keyboard

private extension RedeemVoucherViewController {
    @objc func keyboardWillShow(notification: NSNotification) {
        handleKeyboardOverlapShow(notification: notification)
    }

    @objc func keyboardWillHide() {
        handleKeyboardOverlapHide()
    }

    func handleKeyboardOverlapShow(notification: NSNotification) {
        guard !isViewMoved else { return }

        isViewMoved = true

        navigationControllerOriginY = navigationController?.view.frame.origin.y

        guard let keyboardFrame = (
            notification.userInfo?[UIResponder.keyboardFrameEndUserInfoKey] as? NSValue
        )?.cgRectValue,
            let navigationControllerOriginY = navigationControllerOriginY,
            let navigationController = navigationController else { return }

        let topSafeAreaInset = UIApplication.shared.windows.first?.safeAreaInsets.top ?? 0
        let overlap = navigationControllerOriginY
            + navigationController.view.frame.size.height
            - keyboardFrame.origin.y
        if overlap > 0 {
            let idealNewOrigin = navigationControllerOriginY
                - overlap
                - navigationController.view.frame.origin.x
            navigationController.view.frame.origin.y = idealNewOrigin > topSafeAreaInset
                ? idealNewOrigin
                : topSafeAreaInset
        }
    }

    func handleKeyboardOverlapHide() {
        guard let navigationControllerOriginY = navigationControllerOriginY,
              let navigationController = navigationController,
              isViewMoved else { return }

        isViewMoved = false
        navigationController.view.frame.origin.y = navigationControllerOriginY
    }
}
