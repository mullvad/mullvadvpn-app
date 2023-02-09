package net.mullvad.mullvadvpn.ui.fragment

import android.app.Dialog
import android.content.Context
import android.content.DialogInterface
import android.graphics.drawable.ColorDrawable
import android.os.Bundle
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import android.view.ViewGroup.LayoutParams
import android.widget.Button
import androidx.fragment.app.DialogFragment
import kotlinx.coroutines.CompletableDeferred
import net.mullvad.mullvadvpn.R
import net.mullvad.mullvadvpn.ui.MainActivity

class ConfirmNoEmailDialogFragment : DialogFragment() {
    private var confirmNoEmail: CompletableDeferred<Boolean>? = null

    override fun onAttach(context: Context) {
        super.onAttach(context)

        val parentActivity = context as MainActivity

        confirmNoEmail = parentActivity.problemReport.confirmNoEmail
    }

    override fun onCreateView(
        inflater: LayoutInflater,
        container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        val view = inflater.inflate(R.layout.confirm_no_email, container, false)

        view.findViewById<Button>(R.id.back_button).setOnClickListener {
            activity?.onBackPressed()
        }

        view.findViewById<Button>(R.id.send_button).setOnClickListener {
            confirmNoEmail?.complete(true)
            confirmNoEmail = null
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
        confirmNoEmail?.complete(false)
    }
}
