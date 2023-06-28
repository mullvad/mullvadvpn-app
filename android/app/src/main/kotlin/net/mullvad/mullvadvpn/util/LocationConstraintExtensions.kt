package net.mullvad.mullvadvpn.util

import net.mullvad.mullvadvpn.model.Constraint
import net.mullvad.mullvadvpn.model.GeographicLocationConstraint
import net.mullvad.mullvadvpn.model.LocationConstraint

fun LocationConstraint.toGeographicLocationConstraint(): GeographicLocationConstraint? =
    when(this) {
        is LocationConstraint.Location -> this.location
        is LocationConstraint.CustomList -> null
    }

fun Constraint<LocationConstraint>.toGeographicLocationConstraint(): Constraint<GeographicLocationConstraint> =
    when(this) {
       is Constraint.Only -> when(this.value) {
           is LocationConstraint.Location -> Constraint.Only(this.value.location)
           is LocationConstraint.CustomList -> Constraint.Any()
       }
       is Constraint.Any -> Constraint.Any()
    }



