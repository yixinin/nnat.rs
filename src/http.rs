pub enum HttpMethod {
    MethodGet,
    MethodHead,
    MethodPost,
    MethodPut,
    MethodPatch, // RFC 5789
    MethodDelete,
    MethodConnect,
    MethodOptions,
    MethodTrace,
    MethodUnknown,
}
const METHOD_GET: String = String::from("GET");
const METHOD_HEAD: String = String::from("HEAD");
const METHOD_POST: String = String::from("POST");
const METHOD_PUT: String = String::from("PUT");
const METHOD_PATCH: String = String::from("PATCH");
const METHOD_DELETE: String = String::from("DELETE");
const METHOD_CONNECT: String = String::from("CONNECT");
const METHOD_OPTIONS: String = String::from("OPTIONS");
const METHOD_TRACE: String = String::from("TRACE");

pub fn new(data: &[u8]) -> HttpMethod {
    let s: String = String::from_utf8_lossy(data).to_string();
    match s {
        METHOD_GET => HttpMethod::MethodGet,
        METHOD_HEAD => HttpMethod::MethodHead,
        METHOD_POST => HttpMethod::MethodPost,
        METHOD_PUT => HttpMethod::MethodPut,
        METHOD_PATCH => HttpMethod::MethodPatch,
        METHOD_DELETE => HttpMethod::MethodDelete,
        METHOD_CONNECT => HttpMethod::MethodConnect,
        METHOD_OPTIONS => HttpMethod::MethodOptions,
        METHOD_TRACE => HttpMethod::MethodTrace,
        _ => HttpMethod::MethodUnknown,
    }
}

impl HttpMethod {
    fn to_string(self) -> String {
        match self {
            MethodGet => String::from("GET"),
            MethodHead => String::from("HEAD"),
            MethodPost => String::from("POST"),
            MethodPut => String::from("PUT"),
            MethodPatch => String::from("PATCH"),
            MethodDelete => String::from("DELETE"),
            MethodConnect => String::from("CONNECT"),
            MethodOptions => String::from("OPTIONS"),
            MethodTrace => String::from("TRACE"),
            _ => String::from("Unknown"),
        }
    }
}
