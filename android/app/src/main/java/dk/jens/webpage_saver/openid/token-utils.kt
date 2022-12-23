package dk.jens.webpage_saver.openid

import android.content.Context

const val TOKEN_PREFS_NAME = "tokens"
const val TOKEN_PREFS_ACCESS_TOKEN = "access-token"
const val TOKEN_PREFS_REFRESH_TOKEN = "refresh-token"

fun storeTokens(ctx: Context, access_token: String, refresh_token: String) {
    val prefs = ctx.getSharedPreferences(TOKEN_PREFS_NAME, Context.MODE_PRIVATE)
    val editor = prefs.edit()
    editor.putString(TOKEN_PREFS_ACCESS_TOKEN, access_token)
    editor.putString(TOKEN_PREFS_REFRESH_TOKEN, refresh_token)
    editor.apply()
}

fun getTokens(ctx: Context) : Tokens {
    val prefs = ctx.getSharedPreferences(TOKEN_PREFS_NAME, Context.MODE_PRIVATE)
    if(prefs.contains(TOKEN_PREFS_ACCESS_TOKEN) && prefs.contains(TOKEN_PREFS_REFRESH_TOKEN)) {
        val accessToken = prefs.getString(TOKEN_PREFS_ACCESS_TOKEN, null)
        val refreshToken = prefs.getString(TOKEN_PREFS_REFRESH_TOKEN, null)
        if(accessToken != null && refreshToken != null) {
            return Tokens(accessToken, refreshToken)
        }
    }
    throw NoTokensException()
}

data class Tokens(val access_token: String, val refresh_token: String)

class NoTokensException : Exception()
