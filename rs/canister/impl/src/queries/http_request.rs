use ic_cdk::query;
use ic_http_certification::{HttpRequest, HttpResponse, HttpResponseBuilder};

#[query]
fn http_request(request: HttpRequest) -> HttpResponse {
    let Ok(path) = request.get_path() else {
        return response_from_status_code(404);
    };
    let segments: Vec<_> = path.split('/').skip(1).collect();

    match segments.first() {
        #[cfg(feature = "dapp-radar")]
        Some(&"dapp-radar") => {
            if let Some(response) =
                process_dapp_radar_request(segments, request.get_query().ok().flatten())
            {
                return response;
            }
        }
        _ => {}
    }

    response_from_status_code(404)
}

#[cfg(feature = "dapp-radar")]
pub fn process_dapp_radar_request(
    segments: Vec<&str>,
    qs: Option<String>,
) -> Option<HttpResponse<'static>> {
    use std::str::FromStr;

    if segments.len() != 4 || segments[0] != "dapp-radar" || segments[1] != "aggregated-data" {
        return None;
    }

    let page = qs
        .as_deref()
        .map(querystring::querify)
        .unwrap_or_default()
        .into_iter()
        .find(|(k, _)| *k == "page")
        .map(|(_, v)| usize::from_str(v).unwrap())
        .unwrap_or_default();

    let date_str = segments[2];
    let grouping = segments[3];

    let date_parts: Vec<_> = date_str.split('-').collect();
    if date_parts.len() != 3 {
        return None;
    }

    let Ok(year) = u32::from_str(date_parts[0]) else {
        return None;
    };

    let Ok(month) = u8::from_str(date_parts[1]) else {
        return None;
    };

    let Ok(day) = u8::from_str(date_parts[2]) else {
        return None;
    };

    let data = if grouping == "daily" {
        crate::state::read(|s| {
            s.integrations_data()
                .dapp_radar
                .daily(year, month, day, page)
        })
    } else if grouping == "hourly" {
        crate::state::read(|s| {
            s.integrations_data()
                .dapp_radar
                .hourly(year, month, day, page)
        })
    } else {
        return None;
    };

    let body = serde_json::to_vec(&data).unwrap();

    Some(
        HttpResponseBuilder::new()
            .with_status_code(200.try_into().unwrap())
            .with_headers(vec![
                ("content-type".to_string(), "application/json".to_string()),
                ("content-length".to_string(), body.len().to_string()),
            ])
            .with_body(body)
            .build(),
    )
}

fn response_from_status_code<'a>(status_code: u16) -> HttpResponse<'a> {
    HttpResponseBuilder::new()
        .with_status_code(status_code.try_into().unwrap())
        .build()
}
