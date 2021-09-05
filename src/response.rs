use std::time::Duration;

pub struct Language(String);

impl Language {
    pub fn english() -> Self {
        Language("en".to_owned())
    }
}

pub struct MediaType(String);

impl MediaType {
    pub fn gemini(language: Option<Language>) -> Self {
        match language {
            Some(language) => MediaType(format!("text/gemini; lang={}", language.0).to_owned()),
            None => MediaType("text/gemini".to_owned())
        }
    }
}

pub struct Response(String);

// TODO: Some kind of check to ensure meta is less than 1024 maybe?
// TODO: Mime Media type struct
#[allow(dead_code)]
impl Response {
    // 1x (INPUT)
    // The <META> line is a prompt which should be displayed to the user.

    /// The requested resource accepts a line of textual user input.
    /// The <META> line is a prompt which should be displayed to the user.
    /// The same resource should then be requested again with the user's input included as a
    /// query component. Queries are included in requests as per the usual generic URL definition
    /// in RFC3986, i.e. separated from the path by a ?. Reserved characters used in the user's
    /// input must be "percent-encoded" as per RFC3986, and space characters should also be
    /// percent-encoded.
    pub fn input(prompt: String) -> Response {
        Response(format!("10 {}\r\n", prompt))
    }
    /// As per status code [Self::input], but for use with sensitive input such as passwords.
    /// Clients should present the prompt as per status code 10, but the user's input should not
    /// be echoed to the screen to prevent it being read by "shoulder surfers".
    pub fn sensitive_input(prompt: String) -> Response {
        Response(format!("11 {}\r\n", prompt))
    }

    // 2x (SUCCESS)
    // The <META> line is a MIME media type which applies to the response body.

    /// The request was handled successfully and a response body will follow the response header.
    /// The <META> line is a MIME media type which applies to the response body.
    pub fn success(media_type: MediaType, contents: String) -> Response {
        Response(format!("20 {}\r\n{}", media_type.0, contents))
    }

    // 3x (REDIRECT)
    // <META> is a new URL for the requested resource.

    /// The server is redirecting the client to a new location for the requested resource.
    /// There is no response body. <META> is a new URL for the requested resource.
    /// The URL may be absolute or relative. The redirect should be considered temporary, i.e.
    /// clients should continue to request the resource at the original address and should not
    /// performance convenience actions like automatically updating bookmarks.
    /// There is no response body.
    pub fn redirect_temporary(url: String) -> Response {
        Response(format!("30 {}\r\n", url))
    }

    /// See [Self::redirect_temporary].
    ///
    /// The requested resource should be consistently requested from the new URL provided in future.
    /// Tools like search engine indexers or content aggregators should update their configurations
    /// to avoid requesting the old URL, and end-user clients may automatically update bookmarks,
    /// etc. Note that clients which only pay attention to the initial digit of status codes will
    /// treat this as a temporary redirect. They will still end up at the right place, they just
    /// won't be able to make use of the knowledge that this redirect is permanent, so they'll pay
    /// a small performance penalty by having to follow the redirect each time.
    pub fn redirect_permanent(url: String) -> Response {
        Response(format!("31 {}\r\n", url))
    }

    // 4x (TEMPORARY FAILURE)
    // The contents of <META> may provide additional information on the failure,
    // and should be displayed to human users.

    /// The request has failed. There is no response body. The nature of the failure is temporary,
    /// i.e. an identical request MAY succeed in the future. The contents of <META> may provide
    /// additional information on the failure, and should be displayed to human users.
    pub fn temporary_failure(reason: String) -> Response {
        Response(format!("40 {}\r\n", reason))
    }

    /// See [Self::temporary_failure].
    ///
    /// The server is unavailable due to overload or maintenance. (cf HTTP 503)
    pub fn server_unavailable(reason: String) -> Response {
        Response(format!("41 {}\r\n", reason))
    }

    /// See [Self::temporary_failure].
    ///
    /// A CGI process, or similar system for generating dynamic content, died unexpectedly or timed out.
    pub fn cgi_error(reason: String) -> Response {
        Response(format!("42 {}\r\n", reason))
    }

    /// See [Self::temporary_failure].
    ///
    /// A proxy request failed because the server was unable to successfully complete a transaction
    /// with the remote host. (cf HTTP 502, 504)
    pub fn proxy_error(reason: String) -> Response {
        Response(format!("43 {}\r\n", reason))
    }

    /// See [Self::temporary_failure].
    ///
    /// Rate limiting is in effect. <META> is an integer number of seconds which the client must
    /// wait before another request is made to this server. (cf HTTP 429)
    pub fn slow_down(minimum_time_before_retry_allowed: Duration) -> Response {
        Response(format!("44 {}\r\n", minimum_time_before_retry_allowed.as_secs()))
    }

    // 5x (PERMANENT FAILURE)
    // The contents of <META> may provide additional information on the failure,
    // and should be displayed to human users.

    /// The request has failed. There is no response body. The nature of the failure is permanent,
    /// i.e. identical future requests will reliably fail for the same reason. The contents of
    /// <META> may provide additional information on the failure, and should be displayed to human
    /// users. Automatic clients such as aggregators or indexing crawlers should not repeat this
    /// request.
    pub fn permanent_failure(reason: String) -> Response {
        Response(format!("50 {}\r\n", reason))
    }

    /// See [Self::permanent_failure].
    ///
    /// The requested resource could not be found but may be available in the future. (cf HTTP 404)
    /// (struggling to remember this important status code? Easy: you can't find things hidden
    /// at Area 51!)
    pub fn not_found(reason: String) -> Response {
        Response(format!("51 {}\r\n", reason))
    }

    /// See [Self::permanent_failure].
    ///
    /// The resource requested is no longer available and will not be available again. Search
    /// engines and similar tools should remove this resource from their indices. Content
    /// aggregators should stop requesting the resource and convey to their human users that the
    /// subscribed resource is gone. (cf HTTP 410)
    pub fn gone(reason: String) -> Response {
        Response(format!("52 {}\r\n", reason))
    }

    /// See [Self::permanent_failure].
    ///
    /// The request was for a resource at a domain not served by the server and the server does not
    /// accept proxy requests.
    pub fn proxy_request_refused(reason: String) -> Response {
        Response(format!("53 {}\r\n", reason))
    }

    /// See [Self::permanent_failure].
    ///
    /// The server was unable to parse the client's request, presumably due to a malformed
    /// request. (cf HTTP 400)
    pub fn bad_request(reason: String) -> Response {
        Response(format!("59 {}\r\n", reason))
    }

    // 6x (CLIENT CERTIFICATE REQUIRED)
    // The contents of <META> (and/or the specific 6x code) may provide additional information on
    // certificate requirements or the reason a certificate was rejected.

    /// The requested resource requires a client certificate to access. If the request was made
    /// without a certificate, it should be repeated with one. If the request was made with a
    /// certificate, the server did not accept it and the request should be repeated with a
    /// different certificate. The contents of <META> (and/or the specific 6x code) may provide
    /// additional information on certificate requirements or the reason a certificate was rejected.
    pub fn client_certificate_required(message: String) -> Response {
        Response(format!("60 {}\r\n", message))
    }

    /// See [Self::client_certificate_required].
    ///
    /// The supplied client certificate is not authorised for accessing the particular requested
    /// resource. The problem is not with the certificate itself, which may be authorised for
    /// other resources.
    pub fn certificate_not_authorized(message: String) -> Response {
        Response(format!("61 {}\r\n", message))
    }

    /// See [Self::client_certificate_required].
    ///
    /// The supplied client certificate was not accepted because it is not valid. This indicates a
    /// problem with the certificate in and of itself, with no consideration of the particular
    /// requested resource. The most likely cause is that the certificate's validity start date is
    /// in the future or its expiry date has passed, but this code may also indicate an invalid
    /// signature, or a violation of a X509 standard requirements. The <META> should provide more
    /// information about the exact error.
    pub fn certificate_not_valid(message: String) -> Response {
        Response(format!("62 {}\r\n", message))
    }
}

impl Response {
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}