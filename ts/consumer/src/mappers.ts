import {
  CandidEvent,
  CandidEventsResponse,
} from "./candid/idl";
import type { Event, EventsResponse } from "./types";

export function candidEventsResponse(
  candid: CandidEventsResponse,
): EventsResponse {
  return {
    events: candid.events.map(candidEvent),
    latestEventIndex: candid.latest_event_index[0],
  };
}

function candidEvent(candid: CandidEvent): Event {
  return {
    name: candid.name,
    timestamp: candid.timestamp,
    user: candid.user[0],
    source: candid.source[0],
    payload: Array.isArray(candid.payload)
      ? new Uint8Array(candid.payload)
      : candid.payload,
  };
}
