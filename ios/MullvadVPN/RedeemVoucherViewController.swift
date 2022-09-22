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

    // MARK: - Constants

    private let apiProxy: REST.APIProxy

    // MARK: - Views
    
    private lazy var contentView = RedeemVoucherContentView { [weak self] in
        self?.submitVoucher()
    } cancelAction: { [weak self] in
        self?.dismiss(animated: true)
    }

    // MARK: - Variables

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
                updateViews(with: self.redeemVoucherState, animated: true)
            }
        }
    }

    // MARK: - Lifecycles

    override var preferredStatusBarStyle: UIStatusBarStyle { .lightContent }

    init(
        apiProxy: REST.APIProxy = REST.ProxyFactory.shared.createAPIProxy(),
        didDismissOnSuccess: (() -> Void)? = nil,
        didAddTime: (() -> Void)? = nil
    ) {
        self.apiProxy = apiProxy
        self.didDismissOnSuccess = didDismissOnSuccess
        self.didAddTime = didAddTime
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        setUpContentView()
        addObservers()
        updateViews(with: .initial, animated: false)
    }

    // MARK: - View setup

    private func setUpContentView() {
        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }
}

// MARK: - Private Functions

private extension RedeemVoucherViewController {
    private func addObservers() {
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

    @objc private func textDidChange() {
        updateIsVoucherLengthSatisfied()
        if isVoucherLengthSatisfied {
            dismissKeyboard()
        }
    }

    private func updateIsVoucherLengthSatisfied() {
        isVoucherLengthSatisfied =
            contentView.inputTextField.text?.count == contentView.inputTextField.placeholder?.count
    }

    private func dismissKeyboard() {
        contentView.inputTextField.resignFirstResponder()
    }

    private func setRedeemVoucherState(_ state: RedeemVoucherState, animated: Bool) {
        redeemVoucherState = state
        updateViews(with: state, animated: true)
    }

    private func updateViews(with state: RedeemVoucherState, animated: Bool) {
        if animated {
            UIView.animate(withDuration: 0.8,
                           delay: 0,
                           usingSpringWithDamping: 0.5,
                           initialSpringVelocity: 6.9,
                           options: .curveEaseInOut,
                           animations: {
                self.updateViewsAccordingToState(with: state)
            }) { _ in
                self.updateViewsAnimationCompletion(with: state)
                self.view.layoutIfNeeded()
            }
        } else {
            updateViewsAccordingToState(with: state)
        }
    }
    
    private func updateViewsAccordingToState(with state: RedeemVoucherState) {
        contentView
            .updateViews(state: state,
                         isVoucherLengthSatisfied: isVoucherLengthSatisfied,
                         statusLabelText: state.getStatusLabelText(timeAdded: timeAdded))
    }

    private func updateViewsAnimationCompletion(with state: RedeemVoucherState) {
        if case .success = state {
            contentView
                .redeemedVoucherAnimationDidFinishedWithSuccessfulState { [unowned self] in
                    self.didTapGotIt()
                }
        }
    }

    @objc private func didTapGotIt() {
        (didDismissOnSuccess ?? {})()
        dismiss(animated: true)
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
                    self.setRedeemVoucherState(.success(
                        self.formattedTimeAdded(from: submitVoucherResponse.timeAdded)
                    ), animated: true)
                case .failure:
                    self.setRedeemVoucherState(.failure, animated: true)
                default:
                    break
                }
            }
        }
    }

    private func formattedTimeAdded(from timeAdded: Int) -> String {
        let formatter = DateComponentsFormatter()
        formatter.allowedUnits = [.day]
        formatter.unitsStyle = .full

        return formatter.string(from: Double(timeAdded)) ?? ""
    }
}

// MARK: - Keyboard delegates

private extension RedeemVoucherViewController {
    @objc private func keyboardWillShow(notification: NSNotification) {
        handleKeyboardOverlapShow(notification: notification)
    }

    @objc private func keyboardWillHide() {
        handleKeyboardOverlapHide()
    }

    private func handleKeyboardOverlapShow(notification: NSNotification) {
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

    private func handleKeyboardOverlapHide() {
        guard let navigationControllerOriginY = navigationControllerOriginY,
              let navigationController = navigationController,
              isViewMoved else { return }

        isViewMoved = false
        navigationController.view.frame.origin.y = navigationControllerOriginY
    }
}

// MARK: - Redeem Voucher State

extension RedeemVoucherViewController {
    enum RedeemVoucherState: Equatable {
        case success(String)
        case initial, waiting, failure

        var isWaiting: Bool {
            self == .waiting
        }

        func getStatusLabelText(timeAdded: String) -> String {
            switch self {
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
    }
}
