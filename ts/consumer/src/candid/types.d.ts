import type { Principal } from '@dfinity/principal';
import type { ActorMethod } from '@dfinity/agent';

export interface EventsArgs { 'start' : bigint, 'length' : bigint }
export interface EventsResponse {
  'events' : Array<IndexedEvent>,
  'latest_event_index' : [] | [bigint],
}
export interface IdempotentEvent {
  'source' : [] | [string],
  'name' : string,
  'user' : [] | [string],
  'timestamp' : bigint,
  'payload' : Uint8Array | number[],
  'idempotency_key' : bigint,
}
export interface IndexedEvent {
  'source' : [] | [string],
  'name' : string,
  'user' : [] | [string],
  'timestamp' : bigint,
  'index' : bigint,
  'payload' : Uint8Array | number[],
}
export interface InitArgs {
  'push_events_whitelist' : Array<Principal>,
  'read_events_whitelist' : Array<Principal>,
}
export interface PushEventsArgs { 'events' : Array<IdempotentEvent> }
export interface WhitelistedPrincipals {
  'push' : Array<Principal>,
  'read' : Array<Principal>,
}
export interface _SERVICE {
  'events' : ActorMethod<[EventsArgs], EventsResponse>,
  'push_events' : ActorMethod<[PushEventsArgs], undefined>,
  'whitelisted_principals' : ActorMethod<[], WhitelistedPrincipals>,
}
