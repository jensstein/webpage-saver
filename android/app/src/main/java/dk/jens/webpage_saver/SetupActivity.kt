package dk.jens.webpage_saver

import android.content.BroadcastReceiver
import android.content.Context
import android.content.Intent
import android.content.IntentFilter
import android.os.Bundle
import android.util.Log
import androidx.appcompat.app.AppCompatActivity
import androidx.navigation.findNavController
import androidx.navigation.ui.AppBarConfiguration
import androidx.navigation.ui.navigateUp
import androidx.navigation.ui.setupActionBarWithNavController
import dk.jens.webpage_saver.databinding.ActivitySetupBinding

class SetupActivity : AppCompatActivity() {

    private lateinit var appBarConfiguration: AppBarConfiguration
    private lateinit var binding: ActivitySetupBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        binding = ActivitySetupBinding.inflate(layoutInflater)
        setContentView(binding.root)

        setSupportActionBar(binding.toolbar)

        val navController = findNavController(R.id.nav_host_fragment_content_setup)
        appBarConfiguration = AppBarConfiguration(navController.graph)
        setupActionBarWithNavController(navController, appBarConfiguration)

        val broadcastReceiver = object : BroadcastReceiver() {
            override fun onReceive(context: Context?, resultIntent: Intent?) {
                val b = resultIntent?.getBooleanExtra(OPENID_CONNECTION_BROADCAST_RESULT, false)
                if(b != true) {
                    Log.d(LOGGING_TAG, "Error on authorization: $b")
                    showMessage(binding.root, R.string.error_on_authorization)
                }
            }
        }
        registerReceiver(broadcastReceiver, IntentFilter(OPENID_CONNECTION_BROADCAST_FLAG))
    }

    override fun onSupportNavigateUp(): Boolean {
        val navController = findNavController(R.id.nav_host_fragment_content_setup)
        return navController.navigateUp(appBarConfiguration)
                || super.onSupportNavigateUp()
    }
}
