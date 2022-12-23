package dk.jens.webpage_saver

import android.content.Context
import android.view.View
import com.google.android.material.snackbar.BaseTransientBottomBar
import com.google.android.material.snackbar.Snackbar

const val SHARED_PREFS_NAME = "preferences"
const val BASE_URL_PREF_NAME = "base-url"

fun getBaseUrl(ctx: Context) : String {
    val prefs = ctx.getSharedPreferences(SHARED_PREFS_NAME, Context.MODE_PRIVATE)
    val url = prefs.getString(BASE_URL_PREF_NAME, null)
    if(url != null) {
        return url
    }
    throw NoBaseUrlSetException()
}

fun setBaseUrl(ctx: Context, baseUrl: String) {
    var baseUrl = ensureUrlPrefix(baseUrl)
    val prefs = ctx.getSharedPreferences(SHARED_PREFS_NAME, Context.MODE_PRIVATE)
    val editor = prefs.edit()
    editor.putString(BASE_URL_PREF_NAME, baseUrl)
    editor.apply()
}

class NoBaseUrlSetException : Exception()

fun showMessage(view: View, messageId: Int) {
    val snackbar = Snackbar.make(view, messageId, BaseTransientBottomBar.LENGTH_LONG)
    snackbar.show()
}

fun showMessage(view: View, message: String) {
    val snackbar = Snackbar.make(view, message, BaseTransientBottomBar.LENGTH_LONG)
    snackbar.show()
}

fun ensureUrlPrefix(url: String): String {
    if(!("^https?://.*".toRegex() matches url)) {
        return "https://$url"
    }
    return url
}
