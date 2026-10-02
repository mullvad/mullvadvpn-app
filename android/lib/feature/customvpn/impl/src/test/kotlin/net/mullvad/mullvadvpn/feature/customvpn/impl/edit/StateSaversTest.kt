package net.mullvad.mullvadvpn.feature.customvpn.impl.edit

import androidx.compose.foundation.text.input.TextFieldState
import androidx.compose.runtime.mutableStateListOf
import androidx.compose.runtime.saveable.Saver
import androidx.compose.runtime.saveable.SaverScope
import kotlin.test.assertEquals
import kotlin.test.assertFalse
import kotlin.test.assertNotNull
import kotlin.test.assertTrue
import org.junit.jupiter.api.Test

class StateSaversTest {

    private val testSaverScope = SaverScope { true }

    private fun <T : Any, S : Any> Saver<T, S>.saveAndRestore(value: T): T? {
        val saved = with(this) { testSaverScope.save(value) } ?: return null
        return restore(saved)
    }

    @Test
    fun `ensure FormTextFieldSaver saves and restores FormTextField correctly`() {
        val original =
            FormTextField(
                id = "form-field-id-1",
                state = TextFieldState("192.168.1.1/24"),
                isNewlyAdded = true,
            )

        val restored = FormTextFieldSaver.saveAndRestore(original)

        assertNotNull(restored)
        assertEquals("form-field-id-1", restored.id)
        assertEquals("192.168.1.1/24", restored.state.text.toString())
        assertTrue(restored.isNewlyAdded)
    }

    @Test
    fun `ensure PeerStateSaver saves and restores PeerState with multiple allowed IPs correctly`() {
        val allowedIps =
            mutableStateListOf(
                FormTextField(
                    id = "allowed-ip-1",
                    state = TextFieldState("0.0.0.0/0"),
                    isNewlyAdded = false,
                ),
                FormTextField(
                    id = "allowed-ip-2",
                    state = TextFieldState("::/0"),
                    isNewlyAdded = true,
                ),
            )

        val original =
            PeerState(
                id = "peer-id-1",
                publicKey = TextFieldState("public-key-test-123"),
                allowedIps = allowedIps,
                endpointAddress = TextFieldState("vpn.example.com"),
                endpointPort = TextFieldState("51820"),
                isNewlyAdded = true,
            )

        val restored = PeerStateSaver.saveAndRestore(original)

        assertNotNull(restored)
        assertEquals("peer-id-1", restored.id)
        assertEquals("public-key-test-123", restored.publicKey.text.toString())
        assertEquals("vpn.example.com", restored.endpointAddress.text.toString())
        assertEquals("51820", restored.endpointPort.text.toString())
        assertTrue(restored.isNewlyAdded)

        assertEquals(2, restored.allowedIps.size)
        assertEquals("allowed-ip-1", restored.allowedIps[0].id)
        assertEquals("0.0.0.0/0", restored.allowedIps[0].state.text.toString())
        assertFalse(restored.allowedIps[0].isNewlyAdded)
        assertEquals("allowed-ip-2", restored.allowedIps[1].id)
        assertEquals("::/0", restored.allowedIps[1].state.text.toString())
        assertTrue(restored.allowedIps[1].isNewlyAdded)
    }

    @Test
    fun `ensure FormTextFieldListSaver saves and restores empty list`() {
        val originalList = mutableStateListOf<FormTextField>()

        val restored = FormTextFieldListSaver.saveAndRestore(originalList)

        assertNotNull(restored)
        assertTrue(restored.isEmpty())
    }

    @Test
    fun `ensure FormTextFieldListSaver saves and restores list with multiple items`() {
        val originalList =
            mutableStateListOf(
                FormTextField(
                    id = "id-1",
                    state = TextFieldState("First"),
                    isNewlyAdded = false,
                ),
                FormTextField(
                    id = "id-2",
                    state = TextFieldState("Second"),
                    isNewlyAdded = true,
                ),
            )

        val restored = FormTextFieldListSaver.saveAndRestore(originalList)

        assertNotNull(restored)
        assertEquals(2, restored.size)
        assertEquals("id-1", restored[0].id)
        assertEquals("First", restored[0].state.text.toString())
        assertFalse(restored[0].isNewlyAdded)
        assertEquals("id-2", restored[1].id)
        assertEquals("Second", restored[1].state.text.toString())
        assertTrue(restored[1].isNewlyAdded)
    }

    @Test
    fun `ensure PeerStateListSaver saves and restores list with multiple PeerStates`() {
        val peer1 =
            PeerState(
                id = "peer-list-1",
                publicKey = TextFieldState("pubkey1"),
                isNewlyAdded = true,
            )
        val peer2 =
            PeerState(
                id = "peer-list-2",
                publicKey = TextFieldState("pubkey2"),
                isNewlyAdded = false,
            )
        val originalList = mutableStateListOf(peer1, peer2)

        val restored = PeerStateListSaver.saveAndRestore(originalList)

        assertNotNull(restored)
        assertEquals(2, restored.size)
        assertEquals("peer-list-1", restored[0].id)
        assertEquals("pubkey1", restored[0].publicKey.text.toString())
        assertTrue(restored[0].isNewlyAdded)
        assertEquals("peer-list-2", restored[1].id)
        assertEquals("pubkey2", restored[1].publicKey.text.toString())
        assertFalse(restored[1].isNewlyAdded)
    }
}
