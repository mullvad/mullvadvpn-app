//
//  AccountViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 20/03/2019.
//  Copyright © 2019 Amagicom AB. All rights reserved.
//

import UIKit

class AccountViewController: UIViewController {

    @IBOutlet var accountLabel: UILabel!
    @IBOutlet var expiryLabel: UILabel!

    override func viewDidLoad() {
        super.viewDidLoad()

        accountLabel.text = Account.token

        if let expiryDate = Account.expiry {
            let expiry = AccountExpiry(date: expiryDate)
            
            expiryLabel.text = expiry.formattedDate
        }
    }

    @IBAction func doLogout() {
        Account.logout()

        performSegue(withIdentifier: SegueIdentifier.Account.logout.rawValue, sender: self)
    }
}
