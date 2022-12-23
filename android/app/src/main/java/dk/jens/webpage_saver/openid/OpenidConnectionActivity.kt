package dk.jens.webpage_saver.openid

import android.content.ActivityNotFoundException
import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.net.Uri
import android.os.Build
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import dk.jens.webpage_saver.AUTHORIZATION_BROADCAST_FLAG
import dk.jens.webpage_saver.AUTHORIZATION_BROADCAST_RESULT
import dk.jens.webpage_saver.BuildConfig
import dk.jens.webpage_saver.LOGGING_TAG
import dk.jens.webpage_saver.NoBaseUrlSetException
import dk.jens.webpage_saver.OPENID_CONNECTION_BROADCAST_FLAG
import dk.jens.webpage_saver.OPENID_CONNECTION_BROADCAST_RESULT
import dk.jens.webpage_saver.SetupActivity
import dk.jens.webpage_saver.getBaseUrl
import net.openid.appauth.AuthorizationRequest
import net.openid.appauth.AuthorizationService
import net.openid.appauth.AuthorizationServiceConfiguration
import net.openid.appauth.ResponseTypeValues

class OpenidConnectionActivity : AppCompatActivity() {
    private lateinit var service: AuthorizationService
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        val broadcastReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, resultIntent: Intent?) {
                service.dispose()
                // It doesn't seem to work to use setResult here so I send a broadcast instead
                val intent = Intent(OPENID_CONNECTION_BROADCAST_FLAG)
                val b = resultIntent?.extras?.getBoolean(AUTHORIZATION_BROADCAST_RESULT, false)
                intent.putExtra(OPENID_CONNECTION_BROADCAST_RESULT, b)
                sendBroadcast(intent)
                finish()
            }
        }
        registerReceiver(broadcastReceiver, IntentFilter(AUTHORIZATION_BROADCAST_FLAG))
        authorize()
    }

    private fun authorize() {
        try {
            val baseUrl = getBaseUrl(this)
            val serviceConfig = AuthorizationServiceConfiguration(
                Uri.parse("${baseUrl}/auth/oauth2/authorize"),
                Uri.parse("${baseUrl}/not-used")
            )
            val appHost = "${Build.DEVICE}-${Build.MODEL}-${Build.PRODUCT}-${Build.VERSION.SDK_INT}"
            val clientId = BuildConfig.OPENID_CLIENT_ID
            val authRequest = AuthorizationRequest.Builder(
                serviceConfig,
                clientId,
                ResponseTypeValues.CODE,
                Uri.parse(AUTH_REDIRECT_URL)
            )
                .setAdditionalParameters(mapOf("app_host" to appHost))
                .build()
            service = AuthorizationService(this)
            val authorizationIntent = service.getAuthorizationRequestIntent(authRequest)
            startActivity(authorizationIntent)
        } catch (e: NoBaseUrlSetException) {
            startActivity(Intent(this, SetupActivity::class.java))
        } catch (e: ActivityNotFoundException) {
            Log.e(LOGGING_TAG, "No browser to start authorization flow")
        }
    }

    override fun onPause() {
        service.dispose()
        super.onPause()
    }

    override fun onDestroy() {
        service.dispose()
        super.onDestroy()
    }

    companion object {
        // The scheme cannot include underscores
        const val AUTH_REDIRECT_URL = "dk.jens.webpagesaver://token-callback"
    }
}
