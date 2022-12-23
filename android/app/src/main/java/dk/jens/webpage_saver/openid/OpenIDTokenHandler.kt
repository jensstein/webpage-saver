package dk.jens.webpage_saver.openid

import android.content.Intent
import androidx.appcompat.app.AppCompatActivity
import android.os.Bundle
import android.util.Log
import dk.jens.webpage_saver.AUTHORIZATION_BROADCAST_FLAG
import dk.jens.webpage_saver.AUTHORIZATION_BROADCAST_RESULT
import dk.jens.webpage_saver.LOGGING_TAG
import dk.jens.webpage_saver.MainActivity
import dk.jens.webpage_saver.R
import dk.jens.webpage_saver.databinding.ActivityOpenidTokenHandlerBinding

class OpenIDTokenHandler : AppCompatActivity() {
    private lateinit var binding: ActivityOpenidTokenHandlerBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        binding = ActivityOpenidTokenHandlerBinding.inflate(layoutInflater)
        setContentView(binding.root)

        val uri = intent.data
        if(uri == null) {
            Log.e(LOGGING_TAG, "Data from intent was null: $intent")
            binding.openidTokenHandlerTextview.text = getText(R.string.openid_token_handler_token_extract_error)
        }
        val accessToken = uri?.getQueryParameter("access_token")
        val refreshToken = uri?.getQueryParameter("refresh_token")
        val resultIntent = Intent(AUTHORIZATION_BROADCAST_FLAG)
        if(accessToken != null && refreshToken != null) {
            storeTokens(this, accessToken, refreshToken)
            resultIntent.putExtra(AUTHORIZATION_BROADCAST_RESULT, true)
            sendBroadcast(resultIntent)
            startActivity(Intent(this, MainActivity::class.java))
        } else {
            resultIntent.putExtra(AUTHORIZATION_BROADCAST_RESULT, false)
            sendBroadcast(resultIntent)
            binding.openidTokenHandlerTextview.text = getText(R.string.openid_token_handler_token_extract_error)
        }
    }
}
