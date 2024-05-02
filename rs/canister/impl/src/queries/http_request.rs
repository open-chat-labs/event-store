use ic_cdk::query;
use ic_http_certification::{HttpRequest, HttpResponse};

#[query]
fn http_request(request: HttpRequest) -> HttpResponse {
    let Ok(path) = request.get_path() else {
        return response_from_status_code(404);
    };
    let segments: Vec<_> = path.split('/').skip(1).collect();

    match segments.first() {
        #[cfg(feature = "dapp-radar")]
        Some(&"dapp-radar") => {
            if let Some(response) = process_dapp_radar_request(segments) {
                return response;
            }
        }
        _ => {}
    }

    response_from_status_code(404)
}

#[cfg(feature = "dapp-radar")]
pub fn process_dapp_radar_request(segments: Vec<&str>) -> Option<HttpResponse> {
    use std::str::FromStr;

    if segments.len() != 4 || segments[0] != "dapp-radar" || segments[1] != "aggregated-data" {
        return None;
    }

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
        crate::state::read(|s| s.integrations_data().dapp_radar.daily(year, month, day, 0))
    } else if grouping == "hourly" {
        crate::state::read(|s| s.integrations_data().dapp_radar.hourly(year, month, day, 0))
    } else {
        return None;
    };

    let body = serde_json::to_vec(&data).unwrap();

    Some(HttpResponse {
        status_code: 200,
        headers: vec![
            ("content-type".to_string(), "application/json".to_string()),
            ("content-length".to_string(), body.len().to_string()),
        ],
        body,
        upgrade: None,
    })
}

fn response_from_status_code(status_code: u16) -> HttpResponse {
    HttpResponse {
        status_code,
        headers: Vec::new(),
        body: Vec::new(),
        upgrade: None,
    }
}
