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
            UIImage(resource: .iconAccount).rescaled(by: 24 / 20)
        }

        static var alert: UIImage {
            UIImage(resource: .iconAlert).rescaled(by: 24 / 20)
        }

        static var info: UIImage {
            // the info icon was 18x18 cropped
            UIImage(resource: .iconInfo).resizeImage(targetSize: CGSize(width: 21.5, height: 21.5))
        }

        static var settings: UIImage {
            UIImage(resource: .iconSettings).rescaled(by: 24 / 20)
        }

        static var back: UIImage {
            UIImage(resource: .iconBack)
        }

        static var copy: UIImage {
            UIImage(resource: .iconCopy)
        }

        static var hide: UIImage {
            UIImage(resource: .iconObscure)
        }

        static var reload: UIImage {
            UIImage(resource: .iconReload)
        }

        static var rightArrow: UIImage {
            UIImage(resource: .iconArrow)
        }

        static var show: UIImage {
            UIImage(resource: .iconUnobscure)
        }

        // the close button, which comes we consume in two sizes, both of which come from the same asset

        static var closeSmall: UIImage {
            UIImage(named: "IconClose")!.resizeImage(targetSize: CGSize(width: 19, height: 19))
        }

        static var closeLarge: UIImage {
            UIImage(named: "IconClose")!.resizeImage(targetSize: CGSize(width: 29, height: 29))
        }
    }

    enum CellDecoration {
        static var chevronRight: UIImage {
            UIImage(resource: .iconChevron)
        }

        static var chevronDown: UIImage {
            UIImage(resource: .iconChevronDown)
        }

        static var chevronUp: UIImage {
            UIImage(resource: .iconChevronUp)
        }

        static var externalLink: UIImage {
            UIImage(resource: .iconExtlink)
        }

        static var tick: UIImage {
            UIImage(resource: .iconTickSml)
                .resizeImage(targetSize: CGSize(width: 16, height: 16))
        }
    }

    enum Status {
        static var failure: UIImage { UIImage(resource: .iconFail) }
        static var success: UIImage { UIImage(resource: .iconSuccess) }
    }

    // miscellaneous images
    static var backTransitionMask: UIImage {
        UIImage(resource: .iconBackTransitionMask)
    }

    static var spinner: UIImage {
        UIImage(resource: .iconSpinner)
    }

    static var tick: UIImage {
        UIImage(resource: .iconTickSml)
            .resizeImage(targetSize: CGSize(width: 24, height: 24))
    }

    // a utility function to resize an image by an aspect ratio;
    // used for compensating for scalable assets' nominal sizes being off
    func rescaled(by ratio: CGFloat) -> UIImage {
        resizeImage(targetSize: CGSize(
            width: size.width * ratio,
            height: size.height * ratio
        ))
    }
}
