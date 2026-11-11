package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.snapshots.SnapshotStateList
import androidx.compose.runtime.toMutableStateList

val FormTextFieldSaver =
    Saver<FormTextField, Any>(
        save = { formField ->
            val savedState = with(TextFieldState.Saver) { save(formField.state) }

            listOf(
                formField.id,
                savedState,
                formField.isNewlyAdded,
            )
        },
        restore = { value ->
            val list = value as List<*>
            FormTextField(
                id = list[0] as String,
                state = list[1]?.let { TextFieldState.Saver.restore(it) } ?: TextFieldState(),
                isNewlyAdded = list[2] as Boolean,
            )
        },
    )

val PeerStateSaver =
    Saver<PeerState, Any>(
        save = { peerState ->
            val publicKeySaved = with(TextFieldState.Saver) { save(peerState.publicKey) }
            val endpointAddressSaved =
                with(TextFieldState.Saver) { save(peerState.endpointAddress) }
            val endpointPortSaved = with(TextFieldState.Saver) { save(peerState.endpointPort) }
            val allowedIpsSaved = with(FormTextFieldListSaver) { save(peerState.allowedIps) }

            listOf(
                peerState.id,
                publicKeySaved,
                allowedIpsSaved,
                endpointAddressSaved,
                endpointPortSaved,
                peerState.isNewlyAdded,
            )
        },
        restore = { value ->
            val list = value as List<*>

            PeerState(
                id = list[0] as String,
                publicKey = list[1]?.let { TextFieldState.Saver.restore(it) } ?: TextFieldState(),
                allowedIps =
                    list[2]?.let { FormTextFieldListSaver.restore(it) }
                        ?: mutableStateListOf(FormTextField()),
                endpointAddress =
                    list[3]?.let { TextFieldState.Saver.restore(it) } ?: TextFieldState(),
                endpointPort =
                    list[4]?.let { TextFieldState.Saver.restore(it) } ?: TextFieldState(),
                isNewlyAdded = list[5] as Boolean,
            )
        },
    )

val FormTextFieldListSaver =
    Saver<SnapshotStateList<FormTextField>, Any>(
        save = { stateList ->
            stateList.map { formField -> with(FormTextFieldSaver) { save(formField) } }
        },
        restore = { value ->
            val savedList = value as List<*>

            savedList
                .mapNotNull { savedItem -> savedItem?.let { FormTextFieldSaver.restore(it) } }
                .toMutableStateList()
        },
    )

val PeerStateListSaver =
    Saver<SnapshotStateList<PeerState>, Any>(
        save = { stateList ->
            stateList.map { formField -> with(PeerStateSaver) { save(formField) } }
        },
        restore = { value ->
            val savedList = value as List<*>

            savedList
                .mapNotNull { savedItem -> savedItem?.let { PeerStateSaver.restore(it) } }
                .toMutableStateList()
        },
    )
