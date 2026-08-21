// This Source Code Form is subject to the terms of the GPLv3 License.
// You can obtain a copy of the license at https://www.gnu.org/licenses/gpl-3.0.en.html.
//
// This file incorporates work covered by the following copyright and
// permission notice:
//
//   Copyright (c) Mullvad VPN AB. All rights reserved.
//
// SPDX-License-Identifier: GPL-3.0-only

import Foundation

extension FixedWidthInteger {
    /// Saturating integer multiplication. Computes `self * rhs`, saturating at the numeric bounds
    /// instead of overflowing.
    public func saturatingMultiplication(_ rhs: Self) -> Self {
        let (partialValue, isOverflow) = multipliedReportingOverflow(by: rhs)

        if isOverflow {
            return signum() == rhs.signum() ? .max : .min
        } else {
            return partialValue
        }
    }

    /// Saturating integer addition. Computes `self + rhs`, saturating at the numeric bounds
    /// instead of overflowing.
    public func saturatingAddition(_ rhs: Self) -> Self {
        let (partialValue, isOverflow) = addingReportingOverflow(rhs)

        if isOverflow {
            return partialValue.signum() >= 0 ? .min : .max
        } else {
            return partialValue
        }
    }

    /// Saturating integer subtraction. Computes `self - rhs`, saturating at the numeric bounds
    /// instead of overflowing.
    public func saturatingSubtraction(_ rhs: Self) -> Self {
        let (partialValue, isOverflow) = subtractingReportingOverflow(rhs)

        if isOverflow {
            return partialValue.signum() >= 0 ? .min : .max
        } else {
            return partialValue
        }
    }

    /// Saturating integer exponentiation. Computes `self ** exp`, saturating at the numeric
    /// bounds instead of overflowing.
    public func saturatingPow(_ exp: UInt32) -> Self {
        let result = pow(Double(self), Double(exp))

        if result.isFinite {
            if result <= Double(Self.min) {
                return .min
            } else if result >= Double(Self.max) {
                return .max
            } else {
                return Self(result)
            }
        } else {
            return result.sign == .minus ? Self.min : Self.max
        }
    }
}
