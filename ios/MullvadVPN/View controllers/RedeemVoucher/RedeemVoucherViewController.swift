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
    func redeemVoucherDidSuccess(
        _ controller: RedeemVoucherViewController,
        with response: REST.SubmitVoucherResponse
    )
    func redeemVoucherDidCancel(_ controller: RedeemVoucherViewController)
}

class RedeemVoucherViewController: UIViewController, UINavigationControllerDelegate {
    private let contentView = RedeemVoucherContentView()
    private var voucherTask: Cancellable?
    private var interactor: RedeemVoucherInteractor?

    weak var delegate: RedeemVoucherViewControllerDelegate?

    init(interactor: RedeemVoucherInteractor) {
        super.init(nibName: nil, bundle: nil)
        self.interactor = interactor
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }

    // MARK: - Life Cycle

    override func viewDidLoad() {
        super.viewDidLoad()
        configureUI()
        addActions()
    }

    override func viewWillAppear(_ animated: Bool) {
        super.viewWillAppear(animated)
        becameFirstResponder()
    }

    // MARK: - private functions

    private func becameFirstResponder() {
        guard !contentView.isEditing else { return }
        contentView.isEditing = true
    }

    private func addActions() {
        contentView.redeemAction = { [weak self] code in
            self?.submit(code: code)
        }

        contentView.cancelAction = { [weak self] in
            self?.cancel()
        }
    }

    private func configureUI() {
        view.addSubview(contentView)
        NSLayoutConstraint.activate([
            contentView.topAnchor.constraint(equalTo: view.topAnchor),
            contentView.leadingAnchor.constraint(equalTo: view.leadingAnchor),
            contentView.trailingAnchor.constraint(equalTo: view.trailingAnchor),
            contentView.bottomAnchor.constraint(equalTo: view.bottomAnchor),
        ])
    }

    private func submit(code: String) {
        contentView.state = .verifying
        voucherTask = interactor?.redeemVoucher(code: code, completion: { [weak self] result in
            guard let self else { return }
            switch result {
            case let .success(value):
                contentView.state = .success
                delegate?.redeemVoucherDidSuccess(self, with: value)
            case let .failure(error):
                contentView.state = .failure(error)
            }
        })
    }

    private func cancel() {
        contentView.isEditing = false

        voucherTask?.cancel()

        delegate?.redeemVoucherDidCancel(self)
    }
}
