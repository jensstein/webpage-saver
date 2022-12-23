package dk.jens.webpage_saver

import android.content.Intent
import android.os.Bundle
import androidx.fragment.app.Fragment
import android.view.LayoutInflater
import android.view.View
import android.view.ViewGroup
import dk.jens.webpage_saver.databinding.FragmentSetupBinding
import dk.jens.webpage_saver.openid.OpenidConnectionActivity

class SetupFragment : Fragment() {
    private var _binding: FragmentSetupBinding? = null

    // This property is only valid between onCreateView and
    // onDestroyView.
    private val binding get() = _binding!!

    override fun onCreateView(
        inflater: LayoutInflater, container: ViewGroup?,
        savedInstanceState: Bundle?
    ): View {
        _binding = FragmentSetupBinding.inflate(inflater, container, false)
        return binding.root
    }

    override fun onViewCreated(view: View, savedInstanceState: Bundle?) {
        super.onViewCreated(view, savedInstanceState)

        val context = activity?.applicationContext
        if(context != null) {
            try {
                val baseUrl = getBaseUrl(context)
                binding.setupBaseUrlInput.setText(baseUrl)
            } catch (_ : NoBaseUrlSetException) {}
        }
        binding.buttonConnect.setOnClickListener {
            if(context != null) {
                val text = binding.setupBaseUrlInput.text
                setBaseUrl(context, text.toString())
                startActivity(Intent(context, OpenidConnectionActivity::class.java))
            } else {
                showMessage(binding.root, R.string.setup_connect_button_context_missing)
            }
        }
    }

    override fun onDestroyView() {
        super.onDestroyView()
        _binding = null
    }
}
