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
    private let apiProxy = REST.ProxyFactory.shared.createAPIProxy()
    private let contentView = RedeemVoucherInputContentView()
    private var isViewDidAppearOnce = false

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

        navigationController?.delegate = self

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

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)

        if !isViewDidAppearOnce {
            isViewDidAppearOnce = true

            contentView.textField.becomeFirstResponder()
        }
    }

    private func submitVoucher() {
        guard let voucherCode = contentView.textField.text,
              let accountNumber = TunnelManager.shared.deviceState.accountData?.number
        else { return }

        contentView.state = .verifying

        _ = apiProxy.submitVoucher(
            voucherCode: voucherCode,
            accountNumber: accountNumber,
            retryStrategy: .default
        ) { [weak self] completion in
            guard let self = self else { return }

            switch completion {
            case let .success(response):
                self.contentView.state = .success
                self.notifyDelegateDidRedeemVoucher(response)

            case let .failure(error):
                self.contentView.state = .failure(error)

            case .cancelled:
                self.contentView.state = .initial
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
