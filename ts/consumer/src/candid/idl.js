export const idlFactory = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'remove_events_whitelist' : IDL.Vec(IDL.Principal),
    'push_events_whitelist' : IDL.Vec(IDL.Principal),
    'read_events_whitelist' : IDL.Vec(IDL.Principal),
  });
  const EventsArgs = IDL.Record({ 'start' : IDL.Nat64, 'length' : IDL.Nat64 });
  const IndexedEvent = IDL.Record({
    'source' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'user' : IDL.Opt(IDL.Text),
    'timestamp' : IDL.Nat64,
    'index' : IDL.Nat64,
    'payload' : IDL.Vec(IDL.Nat8),
  });
  const EventsResponse = IDL.Record({
    'events' : IDL.Vec(IndexedEvent),
    'latest_event_index' : IDL.Opt(IDL.Nat64),
    'earliest_event_index_stored' : IDL.Opt(IDL.Nat64),
  });
  const IdempotentEvent = IDL.Record({
    'source' : IDL.Opt(IDL.Text),
    'name' : IDL.Text,
    'user' : IDL.Opt(IDL.Text),
    'timestamp' : IDL.Nat64,
    'payload' : IDL.Vec(IDL.Nat8),
    'idempotency_key' : IDL.Nat,
  });
  const PushEventsArgs = IDL.Record({ 'events' : IDL.Vec(IdempotentEvent) });
  const RemoveEventsArgs = IDL.Record({ 'up_to_inclusive' : IDL.Nat64 });
  const RemoveEventsResponse = IDL.Record({
    'latest_event_index' : IDL.Opt(IDL.Nat64),
    'earliest_event_index_stored' : IDL.Opt(IDL.Nat64),
  });
  return IDL.Service({
    'events' : IDL.Func([EventsArgs], [EventsResponse], ['query']),
    'push_events' : IDL.Func([PushEventsArgs], [], []),
    'remove_events' : IDL.Func([RemoveEventsArgs], [RemoveEventsResponse], []),
  });
};
export const init = ({ IDL }) => {
  const InitArgs = IDL.Record({
    'remove_events_whitelist' : IDL.Vec(IDL.Principal),
    'push_events_whitelist' : IDL.Vec(IDL.Principal),
    'read_events_whitelist' : IDL.Vec(IDL.Principal),
  });
  return [InitArgs];
};
