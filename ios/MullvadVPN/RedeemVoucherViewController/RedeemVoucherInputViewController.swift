//
//  RedeemVoucherViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import UIKit

protocol RedeemVoucherInputViewControllerDelegate: AnyObject {
    func redeemVoucherInputViewController(
        _ controller: RedeemVoucherInputViewController,
        didRedeemVoucherWithResponse response: REST.SubmitVoucherResponse
    )
    func redeemVoucherInputViewControllerDidCancel(_ controller: RedeemVoucherInputViewController)
}

class RedeemVoucherInputViewController: UIViewController, UINavigationControllerDelegate {
    private enum State: Equatable {
        case initial, success, waiting, failure

        var statusText: String? {
            switch self {
            case .initial:
                return nil

            case .success:
                return NSLocalizedString(
                    "REDEEM_VOUCHER_STATUS_SUCCESS",
                    tableName: "RedeemVoucher",
                    value: "Voucher is redeemed.",
                    comment: ""
                )

            case .failure:
                return NSLocalizedString(
                    "REDEEM_VOUCHER_STATUS_FAILURE",
                    tableName: "RedeemVoucher",
                    value: "Voucher code is invalid.",
                    comment: ""
                )

            case .waiting:
                return NSLocalizedString(
                    "REDEEM_VOUCHER_STATUS_WAITING",
                    tableName: "RedeemVoucher",
                    value: "Verifying voucher...",
                    comment: ""
                )
            }
        }
    }

    private let apiProxy = REST.ProxyFactory.shared.createAPIProxy()
    private let contentView = RedeemVoucherInputContentView()
    private var state: State = .initial

    weak var delegate: RedeemVoucherInputViewControllerDelegate?

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    init() {
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()

        setupContentView()
        addObservers()
        updateViews(animated: false)

        navigationController?.delegate = self
    }

    private func setupContentView() {
        contentView.redeemAction = { [weak self] in
            self?.submitVoucher()
        }

        contentView.cancelAction = { [weak self] in
            guard let self = self else { return }

            self.delegate?.redeemVoucherInputViewControllerDidCancel(self)
        }

        view.addSubview(contentView)

        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    private func addObservers() {
        NotificationCenter.default.addObserver(
            self,
            selector: #selector(textDidChange),
            name: UITextField.textDidChangeNotification,
            object: contentView.inputTextField
        )
    }

    @objc private func textDidChange() {
        updateViews(animated: true)
    }

    private func dismissKeyboard() {
        contentView.inputTextField.resignFirstResponder()
    }

    private func setState(_ newState: State, animated: Bool) {
        state = newState
        updateViews(animated: animated)
    }

    private func updateViews(animated: Bool) {
        if state == .waiting {
            contentView.showLoading()
        } else {
            contentView.hideLoading()
        }

        contentView.redeemButton.isEnabled = (state == .initial || state == .failure)
        contentView.statusLabel.alpha = state == .initial ? 0 : 1
        contentView.statusLabel.text = state.statusText
        contentView.statusLabel.textColor = state == .failure ? .dangerColor : .white
    }

    private func submitVoucher() {
        guard let voucherCode = contentView.inputTextField.text,
              let accountNumber = TunnelManager.shared.deviceState.accountData?.number
        else { return }

        setState(.waiting, animated: true)

        _ = apiProxy.submitVoucher(
            voucherCode: voucherCode,
            accountNumber: accountNumber,
            retryStrategy: .default
        ) { completion in
            switch completion {
            case let .success(response):
                self.setState(.success, animated: true)
                self.notifyDelegateDidRedeemVoucher(response)

            case .failure:
                self.setState(.failure, animated: true)

            case .cancelled:
                self.setState(.initial, animated: true)
            }
        }
    }

    private func notifyDelegateDidRedeemVoucher(_ response: REST.SubmitVoucherResponse) {
        delegate?.redeemVoucherInputViewController(self, didRedeemVoucherWithResponse: response)
    }

    // MARK: - UINavigationControllerDelegate

    func navigationController(
        _ navigationController: UINavigationController,
        animationControllerFor operation: UINavigationController.Operation,
        from fromVC: UIViewController,
        to toVC: UIViewController
    ) -> UIViewControllerAnimatedTransitioning? {
        if operation == .push {
            return NavigationControllerFadeAnimator()
        }
        return nil
    }
}
