module.exports = {
    reactStrictMode: true,

    experimental: {
        serverActions: {
            // The default limit is 1mb. This is too little when fetching some webpages.
            // In order to avoid implementing batch uploading, I raise the limit to 10mb.
            // next.js gives the error `413 Body exceeded 1mb limit` when too much data is posted.
            // https://nextjs.org/docs/app/api-reference/next-config-js/serverActions#bodysizelimit
            bodySizeLimit: "10mb",
        }
    }
}
