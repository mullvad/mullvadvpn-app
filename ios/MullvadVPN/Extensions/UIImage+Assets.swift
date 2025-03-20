//
//  UIImage+Assets.swift
//  MullvadVPN
//
//  Created by Andrew Bulhak on 2025-03-06.
//  Copyright © 2025 Mullvad VPN AB. All rights reserved.
//

import UIKit

extension UIImage {
    enum Buttons {
        // Button images we expect as tightly cropped 24x24 images. The SVGs are 20x20 with a 2px border
        static var account: UIImage {
            UIImage(named: "IconAccount")!
//                .resized(to: CGSize(width: 24, height: 24), trimmingBorder: 2)
        }

        static var alert: UIImage {
            UIImage(named: "IconAlert")!
//                .resized(to: CGSize(width: 24, height: 24), trimmingBorder: 2)
        }

        static var info: UIImage {
            UIImage(named: "IconInfo")!
        }

        static var settings: UIImage {
            UIImage(named: "IconSettings")!
//                .resized(to: CGSize(width: 24, height: 24), trimmingBorder: 2)
        }

        static var back: UIImage {
            UIImage(named: "IconBack")!
        }

        static var copy: UIImage {
            UIImage(named: "IconCopy")!
        }

        static var hide: UIImage {
            UIImage(named: "IconObscure")!
        }

        static var reload: UIImage {
            UIImage(named: "IconReload")!
        }

        static var rightArrow: UIImage {
            UIImage(named: "IconArrow")!
        }

        static var show: UIImage {
            UIImage(named: "IconUnobscure")!
        }

        // the close button, which comes we consume in two sizes, both of which come from the same asset. The SVG is 48x48, though with 4 pixels of border

        static var closeSmall: UIImage {
            UIImage(named: "IconClose")!
                .resized(to: CGSize(width: 18, height: 18))
        }

        static var close: UIImage {
            UIImage(named: "IconClose")!
//                .resized(to: CGSize(width: 24, height: 24))
        }
    }

    enum CellDecoration {
        static var chevronRight: UIImage {
            UIImage(named: "IconChevron")!
        }

        static var chevronDown: UIImage {
            UIImage(named: "IconChevronDown")!
        }

        static var chevronUp: UIImage {
            UIImage(named: "IconChevronUp")!
        }

        static var externalLink: UIImage {
            UIImage(named: "IconExtlink")!
        }

        static var tick: UIImage {
            UIImage(named: "IconTickSml")!
//                .resized(to: CGSize(width: 16, height: 16))
        }
    }

    enum Status {
        static var failure: UIImage { UIImage(named: "IconFail")! }
        static var success: UIImage { UIImage(named: "IconSuccess")! }
    }

    // miscellaneous images
    static var backTransitionMask: UIImage {
        UIImage(named: "IconBackTransitionMask")!
    }

    static var spinner: UIImage {
        UIImage(named: "IconSpinner")!
    }

    static var tick: UIImage {
        UIImage(named: "IconTickSml")!
//            .resized(to: CGSize(width: 24, height: 24))
    }
}
