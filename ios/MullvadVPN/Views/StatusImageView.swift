//
//  StatusImageView.swift
//  MullvadVPN
//
//  Created by pronebird on 12/02/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class StatusImageView: UIImageView {
    enum Style: Int {
        case success
        case failure

        fileprivate var image: UIImage? {
            switch self {
            case .success:
                return UIImage(named: "IconSuccess")
            case .failure:
                return UIImage(named: "IconFail")
            }
        }
    }

    var style: Style = .success {
        didSet {
            self.image = style.image
        }
    }

    override var accessibilityValue: String? {
        get {
            switch style {
            case .success:
                return "success"
            case .failure:
                return "fail"
            }
        }

        // swiftlint:disable:next unused_setter_value
        set {
            print("This accessibilityValue property is get only")
        }
    }

    override var intrinsicContentSize: CGSize {
        CGSize(width: 60, height: 60)
    }

    override init(frame: CGRect) {
        super.init(frame: frame)
        image = style.image
    }

    init(style: Style) {
        self.style = style
        super.init(image: style.image)
        image = style.image
        accessibilityIdentifier = .statusImageView
    }

    required init?(coder: NSCoder) {
        super.init(coder: coder)
    }
}
