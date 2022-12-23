package dk.jens.webpage_saver

import android.content.Intent
import android.os.Bundle
import android.util.Log
import android.view.Menu
import android.view.MenuItem
import android.widget.Button
import android.widget.EditText
import androidx.appcompat.app.AppCompatActivity

class MainActivity : AppCompatActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)
        // https://developer.android.com/training/sharing/receive#kotlin
        if(intent?.action == Intent.ACTION_SEND) {
            val text = intent.getStringExtra(Intent.EXTRA_TEXT);
            if(text != null && text != "") {
                startWebBrowser(text)
            } else {
                showMessage(findViewById(R.id.main_activity_layout), R.string.text_missing_from_intent)
            }
        } else {
            val urlInput = findViewById<EditText>(R.id.url_input)
            val urlOpenBtn = findViewById<Button>(R.id.url_open_btn)
            urlOpenBtn.setOnClickListener {
                val url = urlInput.text.toString().trim()
                if (url == "") {
                    showMessage(
                        findViewById(R.id.main_activity_layout),
                        R.string.no_url_provided
                    )
                } else {
                    startWebBrowser(url)
                }
            }
        }
    }

    // TODO: this doesn't work
    override fun onSaveInstanceState(outState: Bundle) {
        val urlInput = findViewById<EditText>(R.id.url_input)
        if(urlInput != null) {
            val url = urlInput.text.toString()
            outState.putString(URL_INPUT_KEY, url)
        }
        super.onSaveInstanceState(outState)
    }

    override fun onRestoreInstanceState(savedInstanceState: Bundle) {
        super.onRestoreInstanceState(savedInstanceState)
        val urlInput = findViewById<EditText>(R.id.url_input)
        val url = savedInstanceState.getString(URL_INPUT_KEY, "")
        if(urlInput != null) {
            urlInput.setText(url)
        } else {
            Log.w(LOGGING_TAG, "Unable to find url input field in MainActivity.onRestoreInstanceState")
            showMessage(findViewById(R.id.main_activity_layout),
                R.string.url_input_field_missing_in_mainactivity)
        }
    }

    override fun onCreateOptionsMenu(menu: Menu?): Boolean {
        menuInflater.inflate(R.menu.main_menu, menu)
        return true
    }

    override fun onOptionsItemSelected(item: MenuItem): Boolean {
        when(item.itemId) {
            R.id.settings_menu_btn -> startActivity(Intent(this, SetupActivity::class.java))
        }
        return super.onOptionsItemSelected(item)
    }

    private fun startWebBrowser(url: String) {
        val url = ensureUrlPrefix(url)
        val webBrowserIntent = Intent(this, WebBrowserActivity::class.java)
        webBrowserIntent.putExtra(WebBrowserActivity.URL_EXTRA, url)
        startActivity(webBrowserIntent)
    }

    companion object {
        const val URL_INPUT_KEY = "url-input"
    }
}
