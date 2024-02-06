//
//  AccountDeletionViewController.swift
//  MullvadVPN
//
//  Created by Mojgan on 2023-07-13.
//  Copyright © 2023 Mullvad VPN AB. All rights reserved.
//

import MullvadTypes
import UIKit

protocol AccountDeletionViewControllerDelegate: AnyObject {
    func deleteAccountDidSucceed(controller: AccountDeletionViewController)
    func deleteAccountDidCancel(controller: AccountDeletionViewController)
}

class AccountDeletionViewController: UIViewController {
    private lazy var contentView: AccountDeletionContentView = {
        let view = AccountDeletionContentView()
        view.delegate = self
        return view
    }()

    weak var delegate: AccountDeletionViewControllerDelegate?
    var interactor: AccountDeletionInteractor

    init(interactor: AccountDeletionInteractor) {
        self.interactor = interactor
        super.init(nibName: nil, bundle: nil)
    }

    required init?(coder: NSCoder) {
        fatalError("init(coder:) has not been implemented")
    }

    override func viewDidLoad() {
        super.viewDidLoad()
        configureUI()
        contentView.viewModel = interactor.viewModel
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

    private func configureUI() {
        view.addConstrainedSubviews([contentView]) {
            contentView.pinEdgesToSuperview(.all())
        }
    }

    private func submit(accountNumber: String) {
        contentView.state = .loading
        Task { [weak self] in
            guard let self else { return }
            do {
                try await interactor.delete(accountNumber: accountNumber)
                self.contentView.state = .initial
                self.delegate?.deleteAccountDidSucceed(controller: self)
            } catch {
                self.contentView.state = .failure(error)
            }
        }
    }
}

extension AccountDeletionViewController: AccountDeletionContentViewDelegate {
    func didTapCancelButton(contentView: AccountDeletionContentView, button: AppButton) {
        contentView.isEditing = false
        delegate?.deleteAccountDidCancel(controller: self)
    }

    func didTapDeleteButton(contentView: AccountDeletionContentView, button: AppButton) {
        switch interactor.validate(input: contentView.lastPartOfAccountNumber) {
        case let .success(accountNumber):
            contentView.isEditing = false
            submit(accountNumber: accountNumber)
        case let .failure(error):
            contentView.state = .failure(error)
        }
    }
}
