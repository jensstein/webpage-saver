package dk.jens.webpage_saver

import okhttp3.OkHttpClient
import retrofit2.Retrofit
import retrofit2.converter.jackson.JacksonConverterFactory
import retrofit2.http.Body
import retrofit2.http.Header
import retrofit2.http.Headers
import retrofit2.http.POST
import java.util.concurrent.TimeUnit

data class FetchBody(val url: String, val html: String)
data class RefreshTokenBody(val refresh_token: String)
data class TokenResponse(val access_token: String, val refresh_token: String)

interface BackendApi {
    @Headers("content-type: application/json")
    @POST("api/fetch?auth-type=oauth2")
    suspend fun fetch(@Body body: FetchBody, @Header("Authorization") authorization: String)

    @Headers("content-type: application/json")
    @POST("api/auth/oauth2/refresh-token")
    suspend fun refreshToken(@Body body: RefreshTokenBody): TokenResponse
}

fun createService(baseUrl: String): BackendApi {
    /*
     * If the server limit for body size is surpassed you get this slightly hard-to-parse error:
     * okhttp3.internal.http2.StreamResetException: stream was reset: NO_ERROR
     * https://stackoverflow.com/questions/58958213/problem-when-uploading-video-stream-was-reset-no-error
     * If you set the protocol to HTTP 1.1 you get an error which is easier to read:
     * retrofit2.HttpException: HTTP 413 Request Entity Too Large
     * The only solution seems to be to either raise the limit or implement streaming on the backend.
     */
    val client: OkHttpClient = OkHttpClient.Builder()
        .connectTimeout(15, TimeUnit.SECONDS)
        .readTimeout(15,TimeUnit.SECONDS).build();
    val retrofit: Retrofit = Retrofit.Builder()
        .baseUrl(baseUrl)
        .addConverterFactory(JacksonConverterFactory.create())
        .client(client)
        .build()
    return retrofit.create(BackendApi::class.java)
}
