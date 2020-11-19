package net.mullvad.mullvadvpn.ui

import android.app.Dialog
import android.content.Context
import android.content.DialogInterface
import android.graphics.drawable.ColorDrawable
import android.os.Bundle
import android.support.v4.app.DialogFragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.view.ViewGroup.LayoutParams
import android.widget.Button
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.R

class ConfirmPublicDnsDialogFragment : DialogFragment() {
    private var confirmPublicDns: CompletableDeferred<Boolean>? = null

    override fun onAttach(context: Context) {
        super.onAttach(context)

        val parentActivity = context as MainActivity

        confirmPublicDns = parentActivity.confirmPublicDnsServer
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.confirm_public_dns, container, false)

        view.findViewById<Button>(R.id.back_button).setOnClickListener {
            activity?.onBackPressed()
        }

        view.findViewById<Button>(R.id.confirm_button).setOnClickListener {
            confirmPublicDns?.complete(true)
            confirmPublicDns = null
            dismiss()
        }

        return view
    }

    override fun onCreateDialog(savedInstanceState: Bundle?): Dialog {
        val dialog = super.onCreateDialog(savedInstanceState)

        dialog.window?.setBackgroundDrawable(ColorDrawable(android.R.color.transparent))

        return dialog
    }

    override fun onStart() {
        super.onStart()

        dialog?.window?.setLayout(LayoutParams.MATCH_PARENT, LayoutParams.WRAP_CONTENT)
    }

    override fun onDismiss(dialogInterface: DialogInterface) {
        confirmPublicDns?.complete(false)
    }
}
