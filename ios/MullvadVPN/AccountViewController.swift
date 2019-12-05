//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import Combine
import UIKit

class AccountViewController: UIViewController {

    @IBOutlet var accountLabel: UILabel!
    @IBOutlet var expiryLabel: UILabel!

    private var logoutSubscriber: AnyCancellable?

    override func viewDidLoad() {
        super.viewDidLoad()

        updateView()
    }

    // MARK: - Actions

    @IBAction func doBuyCredits() {
        UIApplication.shared.open(WebLinks.purchaseURL, options: [:])
    }

    @IBAction func doLogout() {
        logoutSubscriber = Account.shared.logout()
            .receive(on: DispatchQueue.main)
            .sink(receiveCompletion: { (_) in
                self.performSegue(withIdentifier: SegueIdentifier.Account.logout.rawValue, sender: self)
            })
    }

    // MARK: - Private

    private func updateView() {
        accountLabel.text = Account.shared.token

        if let expiryDate = Account.shared.expiry {
            let accountExpiry = AccountExpiry(date: expiryDate)

            if accountExpiry.isExpired {
                expiryLabel.text = NSLocalizedString("OUT OF TIME", tableName: "Settings", comment: "")
                expiryLabel.textColor = .dangerColor
            } else {
                expiryLabel.text = accountExpiry.formattedDate
                expiryLabel.textColor = .white
            }
        }
    }
}
