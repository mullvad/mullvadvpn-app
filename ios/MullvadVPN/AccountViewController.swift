//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Mullvad VPN AB. All rights reserved.
//

import Combine
import UIKit

class AccountViewController: UIViewController {

    @IBOutlet var accountTokenButton: UIButton!
    @IBOutlet var expiryLabel: UILabel!

    private var logoutSubscriber: AnyCancellable?
    private var copyToPasteboardSubscriber: AnyCancellable?

    override func viewDidLoad() {
        super.viewDidLoad()

        accountTokenButton.setTitle(Account.shared.token, for: .normal)

        if let expiryDate = Account.shared.expiry {
            let accountExpiry = AccountExpiry(date: expiryDate)

            if accountExpiry.isExpired {
                expiryLabel.text = NSLocalizedString("OUT OF TIME", comment: "")
                expiryLabel.textColor = .dangerColor
            } else {
                expiryLabel.text = accountExpiry.formattedDate
                expiryLabel.textColor = .white
            }
        }
    }

    // MARK: - Actions

    @IBAction func doLogout() {
        let message = NSLocalizedString("Logging out. Please wait...",
                                        comment: "A modal message displayed during log out")

        let alertController = UIAlertController(
            title: nil,
            message: message,
            preferredStyle: .alert)

        present(alertController, animated: true) {
            self.logoutSubscriber = Account.shared.logout()
                .delay(for: .seconds(1), scheduler: DispatchQueue.main)
                .sink(receiveCompletion: { (completion) in
                    switch completion {
                    case .failure(let error):
                        alertController.dismiss(animated: true) {
                            self.presentError(error, preferredStyle: .alert)
                        }

                    case .finished:
                        self.performSegue(withIdentifier: SegueIdentifier.Account.logout.rawValue, sender: self)
                    }
                })
        }
    }

    @IBAction func copyAccountToken() {
        UIPasteboard.general.string = Account.shared.token

        accountTokenButton.setTitle(
            NSLocalizedString("COPIED TO PASTEBOARD!", comment: ""),
            for: .normal)

        copyToPasteboardSubscriber =
            Just(()).delay(for: .seconds(3), scheduler: DispatchQueue.main)
                .sink(receiveValue: { _ in
                    self.accountTokenButton.setTitle(Account.shared.token, for: .normal)
                })
    }

}
