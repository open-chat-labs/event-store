import {
  CandidEvent,
  CandidEventsResponse,
  CandidRemoveEventsResponse,
} from "./candid/idl";
import type { Event, EventsResponse, RemoveEventsResponse } from "./types";

export function candidEventsResponse(
  candid: CandidEventsResponse,
): EventsResponse {
  return {
    events: candid.events.map(candidEvent),
    latestEventIndex: candid.latest_event_index[0],
    earliestEventIndexStored: candid.earliest_event_index_stored[0],
  };
}

export function candidRemoveEventsResponse(
  candid: CandidRemoveEventsResponse,
): RemoveEventsResponse {
  return {
    latestEventIndex: candid.latest_event_index[0],
    earliestEventIndexStored: candid.earliest_event_index_stored[0],
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
