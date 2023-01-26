package dk.jens.webpage_saver

import android.annotation.SuppressLint
import android.content.Context
import android.content.Intent
import android.os.Bundle
import android.os.Handler
import android.os.Looper
import android.util.JsonReader
import android.util.JsonToken
import android.util.Log
import android.view.View
import android.webkit.ConsoleMessage
import android.webkit.CookieManager
import android.webkit.WebChromeClient
import android.webkit.WebView
import android.webkit.WebViewClient
import androidx.appcompat.app.AppCompatActivity
import dk.jens.webpage_saver.databinding.ActivityWebbrowserBinding
import dk.jens.webpage_saver.openid.NoTokensException
import dk.jens.webpage_saver.openid.OpenidConnectionActivity
import dk.jens.webpage_saver.openid.Tokens
import dk.jens.webpage_saver.openid.getTokens
import dk.jens.webpage_saver.openid.storeTokens
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.launch
import retrofit2.HttpException
import java.io.IOException
import java.io.StringReader
import java.net.MalformedURLException
import java.net.SocketTimeoutException
import java.net.URL

class WebBrowserActivity : AppCompatActivity() {
    private lateinit var binding: ActivityWebbrowserBinding

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)

        binding = ActivityWebbrowserBinding.inflate(layoutInflater)
        setContentView(binding.root)

        setSupportActionBar(binding.toolbar)
        openPage()
    }

    private fun openPage() {
        try {
            val tokens = getTokens(this)
            val baseUrl = getBaseUrl(this)

            val url = intent.getStringExtra(URL_EXTRA)
            if (url != null && url != "") {
                val webview = binding.webbrowserLayout.webview
                CookieManager.getInstance().setAcceptCookie(true)
                CookieManager.getInstance().setAcceptThirdPartyCookies(webview, true)
                @SuppressLint("SetJavaScriptEnabled")
                webview.settings.javaScriptEnabled = true
                webview.loadUrl(url)
                if(BuildConfig.DEBUG) {
                    // https://stackoverflow.com/a/40485201
                    webview.webChromeClient = object : WebChromeClient() {
                        override fun onConsoleMessage(consoleMessage: ConsoleMessage): Boolean {
                            Log.i(LOGGING_TAG,
                                "webview console: ${consoleMessage.message()} (${consoleMessage.sourceId()}:${consoleMessage.lineNumber()})")
                            return super.onConsoleMessage(consoleMessage);
                        }
                    }
                }
                webview.webViewClient = object : WebViewClient() {
                    override fun onPageFinished(view: WebView?, url: String?) {
                        super.onPageFinished(view, url)
                        val fallbackUrl = url ?: "https://url-missing"
                        val url = URL(view?.url ?: fallbackUrl)
                        binding.fetchPageBtn.visibility = View.VISIBLE
                        binding.fetchPageBtn.setOnClickListener {
                            // https://stackoverflow.com/a/32040564
                            view?.evaluateJavascript(
                                "(function() { console.log('Serializing document'); return new XMLSerializer().serializeToString(document); })();"
                            ) { html ->
                                val reader = JsonReader(StringReader(html))
                                reader.isLenient = true
                                try {
                                    if (reader.peek() == JsonToken.STRING) {
                                        /*
                                         * `serializeToString` produces some
                                         * escaped characters when run from a webview
                                         * that doesn't occur when running in a normal browser context.
                                         * < becomes \u003C and newlines are escaped.
                                         * If you parse the html as one json string these
                                         * escaped characters are all unescaped.
                                         * https://stackoverflow.com/a/54139817
                                         */
                                        val html = reader.nextString()
                                        val page = Page(html, url)
                                        fetchPage(this@WebBrowserActivity, baseUrl, tokens, page)
                                    } else {
                                        Log.w(LOGGING_TAG, "Unexpected json representation of html: ${reader.peek()}")
                                        showMessage(binding.root, R.string.unexpected_json_representation_of_html)
                                    }
                                } catch(e: MalformedURLException) {
                                    Log.w(LOGGING_TAG, "Malformed URL: $e")
                                    showMessage(binding.root, this@WebBrowserActivity.getString(R.string.malformed_url_input, e))
                                } catch(e: IOException) {
                                    Log.e(LOGGING_TAG, "error parsing html as json: $e")
                                    showMessage(binding.root, R.string.error_parsing_page_source_as_json)
                                } finally {
                                    reader.close()
                                }
                            }
                        }
                    }
                }
            } else {
                showMessage(binding.root, R.string.no_url_provided)
                Log.w(LOGGING_TAG, "URL is null or empty when given to the web browser: $url")
                // Wait 3 seconds before finishing the activity to allow the message to be viewed
                Handler(Looper.getMainLooper()).postDelayed({
                    finish()
                }, 3000)
            }
        } catch (e: NoTokensException) {
            showMessage(binding.root, R.string.no_tokens_found)
            startActivity(Intent(this, OpenidConnectionActivity::class.java))
        } catch (e: NoBaseUrlSetException) {
            showMessage(binding.root, R.string.no_base_url_found)
            startActivity(Intent(this, SetupActivity::class.java))
        }
    }

    private suspend fun fetch(context: Context, service: BackendApi, page: Page, access_token: String) {
        service.fetch(FetchBody(page.url.toString(), page.content), "bearer $access_token")
        val displayUrl = if (page.url.toString().length > 10) {
            val urlSubparts = "${page.url.host}/${page.url.path}"
            "${urlSubparts.substring(0, 10)}..."
        } else {
            page.url.toString()
        }
        showMessage(binding.root, context.getString(R.string.fetch_page_success, displayUrl))
    }

    private fun fetchPage(context: Context, baseUrl: String, tokens: Tokens, page: Page) {
        val service = createService(baseUrl)
        GlobalScope.launch {
            try {
                fetch(context, service, page, tokens.access_token)
            } catch (e: HttpException) {
                if (e.code() == 401) {
                    try {
                        val tokensResponse = service.refreshToken(RefreshTokenBody(tokens.refresh_token))
                        if(tokensResponse.isSuccessful) {
                            val newTokens = tokensResponse.body()
                            if(newTokens != null) {
                                storeTokens(context, newTokens.access_token, newTokens.refresh_token)
                                fetch(context, service, page, newTokens.access_token)
                            } else {
                                startActivity(Intent(context, OpenidConnectionActivity::class.java))
                            }
                        } else {
                            startActivity(Intent(context, OpenidConnectionActivity::class.java))
                        }
                    } catch (e2: HttpException) {
                        if (e2.code() == 401) {
                            startActivity(Intent(context, OpenidConnectionActivity::class.java))
                        } else {
                            showMessage(
                                binding.root, this@WebBrowserActivity.getString(
                                    R.string.error_on_token_refresh, e2
                                )
                            )
                        }
                    }
                } else {
                    showMessage(
                        binding.root, this@WebBrowserActivity.getString(
                            R.string.error_on_page_fetch, e
                        )
                    )
                }
            } catch (e: SocketTimeoutException) {
                Log.d(LOGGING_TAG, "Timeout connecting to backend: $e")
                showMessage(binding.root, R.string.timeout_connecting_to_backend)
            }
        }
    }

    data class Page(val content: String, val url: URL)

    companion object {
        const val URL_EXTRA = "url"
    }
}
