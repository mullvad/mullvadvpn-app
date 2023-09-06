//
//  RedeemVoucherViewController.swift
//  MullvadVPN
//
//  Created by Andreas Lif on 2022-08-05.
//  Copyright © 2022 Mullvad VPN AB. All rights reserved.
//

import MullvadREST
import MullvadTypes
import UIKit

protocol RedeemVoucherViewControllerDelegate: AnyObject {
    func redeemVoucherDidSucceed(
        _ controller: RedeemVoucherViewController,
        with response: REST.SubmitVoucherResponse
    )
    func redeemVoucherDidCancel(_ controller: RedeemVoucherViewController)
    func viewForInputingAccountNumberAsVoucher(_ controller: RedeemVoucherViewController) -> UIView?
}

class RedeemVoucherViewController: UIViewController, UINavigationControllerDelegate, RootContainment {
    private let contentView = RedeemVoucherContentView()
    private var interactor: RedeemVoucherProtocol?

    weak var delegate: RedeemVoucherViewControllerDelegate?

    init(interactor: RedeemVoucherProtocol) {
        super.init(nibName: nil, bundle: nil)
        self.interactor = interactor
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        .lightContent
    }

    var preferredHeaderBarPresentation: HeaderBarPresentation {
        HeaderBarPresentation(style: .default, showsDivider: true)
    }

    var prefersHeaderBarHidden: Bool {
        false
    }

    var prefersDeviceInfoBarHidden: Bool {
        true
    }

    var prefersNotificationBarHidden: Bool {
        true
    }

    // MARK: - Life Cycle

    override func viewDidLoad() {
        super.viewDidLoad()
        configureUI()
        addActions()
    }

    override func viewDidAppear(_ animated: Bool) {
        super.viewDidAppear(animated)
        contentView.isEditing = true
    }

    override func viewWillDisappear(_ animated: Bool) {
        super.viewWillDisappear(animated)
        contentView.isEditing = false
    }

    override func viewWillTransition(to size: CGSize, with coordinator: UIViewControllerTransitionCoordinator) {
        contentView.isEditing = false
        super.viewWillTransition(to: size, with: coordinator)
    }

    // MARK: - private functions

    private func addActions() {
        contentView.redeemAction = { [weak self] code in
            self?.submit(code: code)
        }

        contentView.cancelAction = { [weak self] in
            self?.cancel()
        }
    }

    private func configureUI() {
        view.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview(.all())
        }
        contentView.viewForInputingAccountNumberAsVoucher = delegate?.viewForInputingAccountNumberAsVoucher(self)
    }

    private func submit(code: String) {
        contentView.state = .verifying
        contentView.isEditing = false
        interactor?.redeemVoucher(code: code, completion: { [weak self] result in
            guard let self else { return }
            switch result {
            case let .success(value):
                contentView.state = .success
                delegate?.redeemVoucherDidSucceed(self, with: value)
            case let .failure(error):
                contentView.state = .failure(error)
            }
        })
    }

    private func cancel() {
        contentView.isEditing = false

        interactor?.cancelAll()

        delegate?.redeemVoucherDidCancel(self)
    }
}
